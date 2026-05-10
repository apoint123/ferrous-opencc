use std::{
    fs::File,
    io::{
        BufReader,
        Read,
        Write,
    },
    path::Path,
};

use ferrous_opencc_compiler::ArchivedSerializableFstDict;
use fst::Map;

use crate::{
    dictionary::Dictionary,
    error::Result,
};

#[derive(Debug)]
pub struct FstDict {
    map: Map<Vec<u8>>,
    metadata_bytes: Vec<u8>,
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
        let mut file = File::create(path)?;
        let mut final_bytes = Vec::new();
        final_bytes.write_all(&(self.metadata_bytes.len() as u64).to_le_bytes())?;
        final_bytes.write_all(&self.metadata_bytes)?;
        final_bytes.write_all(self.map.as_fst().as_bytes())?;

        file.write_all(&final_bytes)?;
        Ok(())
    }

    pub fn from_text(path: &Path) -> Result<Self> {
        let ocb_bytes = ferrous_opencc_compiler::compile_dictionary(path)?;
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

        rkyv::access::<ArchivedSerializableFstDict, rkyv::rancor::Error>(&metadata_bytes)?;

        let mut fst_bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut fst_bytes)?;

        let map = Map::new(fst_bytes)?;

        Ok(Self {
            map,
            metadata_bytes,
        })
    }
}

impl Dictionary for FstDict {
    fn match_prefix<'a, 'b>(&'a self, word: &'b str) -> Option<(&'b str, &'a str)> {
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

        if let Some((len, value_index)) = last_match {
            let metadata = unsafe {
                rkyv::access_unchecked::<ArchivedSerializableFstDict>(&self.metadata_bytes)
            };

            if let Some(values) = metadata.values.get(value_index as usize)
                && let Some(first_value) = values.iter().next()
            {
                let key = &word[..len];
                return Some((key, first_value.as_str()));
            }
        }

        None
    }

    fn max_key_length(&self) -> usize {
        let metadata =
            unsafe { rkyv::access_unchecked::<ArchivedSerializableFstDict>(&self.metadata_bytes) };

        rkyv::deserialize::<u32, rkyv::rancor::Error>(&metadata.max_key_length).unwrap() as usize
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

        let (key, value) = dict.match_prefix("一个").unwrap();
        assert_eq!(key, "一个");
        assert_eq!(value, "一個");
        let (key, value) = dict.match_prefix("一个半小时").unwrap();
        assert_eq!(key, "一个半");
        assert_eq!(value, "一個半");

        let (key, value) = dict.match_prefix("世纪之交").unwrap();
        assert_eq!(key, "世纪");
        assert_eq!(value, "世紀");
        let (key, value) = dict.match_prefix("一").unwrap();
        assert_eq!(key, "一");
        assert_eq!(value, "一");
        let (key, value) = dict.match_prefix("一").unwrap();
        assert_eq!(key, "一");
        assert_eq!(value, "一");
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

        let (key, value) = dict_from_ocb.match_prefix("你好世界").unwrap();
        assert_eq!(key, "你好");
        assert_eq!(value, "Hello");

        let (key, value) = dict_from_ocb.match_prefix("你好世界").unwrap();
        assert_eq!(key, "你好");
        assert_eq!(value, "Hello");
    }
}
