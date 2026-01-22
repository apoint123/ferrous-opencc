use std::{
    fs::File,
    io::{
        BufReader,
        Read,
        Write,
    },
    path::Path,
};

use bincode::config;
use ferrous_opencc_compiler::{
    Delta,
    SerializableFstDict,
    compile_dictionary,
};
use fst::Map;

use crate::{
    dictionary::Dictionary,
    error::Result,
};

#[derive(Debug)]
pub struct FstDict {
    map: Map<Vec<u8>>,
    values: Vec<Vec<Delta>>,
    max_key_length: usize,
}

fn apply_delta(key: &str, delta: &Delta) -> String {
    match delta {
        Delta::FullReplacement(s) => s.to_string(),
        Delta::CharDiffs(diffs) => {
            let mut chars: Vec<char> = key.chars().collect();
            for &(index, new_char) in diffs {
                if let Some(c) = chars.get_mut(index as usize) {
                    *c = new_char;
                }
            }
            chars.into_iter().collect()
        }
    }
}

impl FstDict {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let compiled_path = path.with_extension("ocb");

        if compiled_path.is_file() {
            if let Ok(text_meta) = path.metadata() {
                if let Ok(compiled_meta) = compiled_path.metadata() {
                    let text_modified = text_meta.modified()?;
                    let compiled_modified = compiled_meta.modified()?;
                    if compiled_modified > text_modified {
                        return Self::from_ocb_file(&compiled_path);
                    }
                }
            } else {
                return Self::from_ocb_file(&compiled_path);
            }
        }

        let dict = Self::from_text(path)?;
        let _ = dict.serialize_to_file(&compiled_path);
        Ok(dict)
    }

    fn from_ocb_file(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }

    pub fn serialize_to_file(&self, path: &Path) -> Result<()> {
        let values_bytes = bincode::encode_to_vec(&self.values, config::standard())?;
        let compressed_values = zstd::encode_all(&values_bytes[..], 0)?;

        let metadata = SerializableFstDict {
            compressed_values,
            max_key_length: self.max_key_length,
        };
        let metadata_bytes = bincode::encode_to_vec(&metadata, config::standard())?;

        let mut final_bytes = Vec::new();
        final_bytes.write_all(&(metadata_bytes.len() as u64).to_le_bytes())?;
        final_bytes.write_all(&metadata_bytes)?;
        final_bytes.write_all(self.map.as_fst().as_bytes())?;

        let mut file = File::create(path)?;
        file.write_all(&final_bytes)?;

        Ok(())
    }

    pub fn from_text(path: &Path) -> Result<Self> {
        let ocb_bytes = compile_dictionary(path)?;
        Self::from_ocb_bytes(&ocb_bytes)
    }

    pub fn from_ocb_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_reader(bytes)
    }

    fn from_reader<R: Read>(mut reader: R) -> Result<Self> {
        let mut len_bytes = [0u8; 8];
        reader.read_exact(&mut len_bytes)?;
        let metadata_len = u64::from_le_bytes(len_bytes) as usize;

        let mut metadata_bytes = vec![0; metadata_len];
        reader.read_exact(&mut metadata_bytes)?;

        let (metadata, _): (SerializableFstDict, usize) =
            bincode::decode_from_slice(&metadata_bytes, config::standard())?;

        let values_bytes = zstd::decode_all(&metadata.compressed_values[..])?;
        let (values, _): (Vec<Vec<Delta>>, usize) =
            bincode::decode_from_slice(&values_bytes, config::standard())?;

        let mut fst_bytes = Vec::new();
        reader.read_to_end(&mut fst_bytes)?;

        let map = Map::new(fst_bytes)?;

        Ok(Self {
            map,
            values,
            max_key_length: metadata.max_key_length,
        })
    }
}

impl Dictionary for FstDict {
    fn match_prefix<'a>(&self, word: &'a str) -> Option<(&'a str, Vec<String>)> {
        let fst = self.map.as_fst();
        let mut node = fst.root();

        let mut last_match: Option<(usize, u64)> = None;

        let mut current_output: u64 = 0;

        for (i, byte) in word.as_bytes().iter().enumerate() {
            if let Some(trans_idx) = node.find_input(*byte) {
                let t = node.transition(trans_idx);
                current_output += t.out.value();
                node = fst.node(t.addr);

                if node.is_final() {
                    let final_value = current_output + node.final_output().value();
                    last_match = Some((i + 1, final_value));
                }
            } else {
                break;
            }
        }

        if let Some((len, value_index)) = last_match
            && let Some(deltas) = self.values.get(value_index as usize)
        {
            let key = &word[..len];
            let result_values: Vec<String> =
                deltas.iter().map(|delta| apply_delta(key, delta)).collect();
            return Some((key, result_values));
        }

        None
    }

    fn max_key_length(&self) -> usize {
        self.max_key_length
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;

    fn create_test_dict_file(dir: &tempfile::TempDir, content: &str) -> PathBuf {
        let dict_path = dir.path().join("test_dict.txt");
        let mut file = File::create(&dict_path).unwrap();
        writeln!(file, "{content}").unwrap();
        dict_path
    }

    #[test]
    fn test_from_text_and_match_prefix() {
        let dir = tempdir().unwrap();
        let dict_content = "一\t一\n一个\t一個\n一个半\t一個半\n世纪\t世紀";
        let dict_path = create_test_dict_file(&dir, dict_content);

        let dict = FstDict::from_text(&dict_path).unwrap();

        let (key, values) = dict.match_prefix("一个").unwrap();
        assert_eq!(key, "一个");
        let values_str: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
        assert_eq!(values_str, ["一個"]);
        let (key, values) = dict.match_prefix("一个半小时").unwrap();
        assert_eq!(key, "一个半");
        let values_str: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
        assert_eq!(values_str, ["一個半"]);
        let values_str: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
        assert_eq!(values_str, ["一個半"]);

        let (key, values) = dict.match_prefix("世纪之交").unwrap();
        assert_eq!(key, "世纪");
        let values_str: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
        assert_eq!(values_str, ["世紀"]);
        let (key, values) = dict.match_prefix("一").unwrap();
        assert_eq!(key, "一");
        let values_str: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
        assert_eq!(values_str, ["一"]);
        let (key, values) = dict.match_prefix("一").unwrap();
        assert_eq!(key, "一");
        let values_str: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
        assert_eq!(values_str, ["一"]);
    }

    #[test]
    fn test_serialization_and_deserialization() {
        let dir = tempdir().unwrap();
        let dict_content = "你好\tHello\n世界\tWorld";
        let txt_path = create_test_dict_file(&dir, dict_content);
        let ocb_path = txt_path.with_extension("ocb");

        let dict_from_text = FstDict::from_text(&txt_path).unwrap();
        dict_from_text.serialize_to_file(&ocb_path).unwrap();

        assert!(ocb_path.exists());

        let dict_from_ocb = FstDict::new(&txt_path).unwrap();

        let (key, values) = dict_from_ocb.match_prefix("你好世界").unwrap();
        assert_eq!(key, "你好");
        let values_str: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
        assert_eq!(values_str, ["Hello"]);

        let (key, values) = dict_from_ocb.match_prefix("你好世界").unwrap();
        assert_eq!(key, "你好");
        let values_str: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
        assert_eq!(values_str, ["Hello"]);
    }
}
