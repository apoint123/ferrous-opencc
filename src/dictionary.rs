use std::{
    fmt::Debug,
    fs::File,
    io::{
        Read,
        Write,
    },
    path::Path,
};

use fst::Map;
use rkyv::{
    Archive,
    Deserialize,
    Serialize,
};

use crate::error::Result;

#[cfg(feature = "compress")]
const COMPRESSION_MAGIC: &[u8; 4] = b"CMP\0";

pub trait Dictionary: Send + Sync + Debug {
    fn match_prefix<'a, 'b>(&'a self, config_id: u8, word: &'b str) -> Option<(&'b str, &'a str)>;
}

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct SerializableFstDict {
    pub values: Vec<Vec<String>>,
}

#[derive(Debug)]
pub struct FstDict {
    map: Map<Vec<u8>>,
    metadata_bytes: Vec<u8>,
}

impl FstDict {
    #[cfg(feature = "runtime-compilation")]
    pub fn from_text(path: &std::path::Path) -> Result<Self> {
        let single_dict_chain = ferrous_opencc_compiler::CompilerChain {
            groups: vec![ferrous_opencc_compiler::CompilerDictGroup {
                dict_paths: vec![path.to_path_buf()],
            }],
        };

        let ocb_bytes = ferrous_opencc_compiler::compile_global_dictionary(&[(
            crate::DYNAMIC_CONFIG_ID,
            single_dict_chain,
        )])
        .map_err(|e| crate::error::OpenCCError::InvalidConfig(e.to_string()))?;

        Self::from_ocb_bytes(&ocb_bytes)
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

    pub fn from_ocb_bytes(bytes: &[u8]) -> Result<Self> {
        #[cfg(feature = "compress")]
        {
            if bytes.starts_with(COMPRESSION_MAGIC) {
                let compressed = &bytes[4..];
                let mut decoder =
                    ruzstd::decoding::StreamingDecoder::new(compressed).map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                    })?;
                let mut decompressed = Vec::new();
                std::io::Read::read_to_end(&mut decoder, &mut decompressed)?;
                return Self::from_reader(decompressed.as_slice());
            }
        }
        Self::from_reader(bytes)
    }

    fn from_reader<R: Read>(mut reader: R) -> Result<Self> {
        let mut len_bytes = [0u8; 8];
        reader.read_exact(&mut len_bytes)?;
        let metadata_len = usize::try_from(u64::from_le_bytes(len_bytes)).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "metadata length too large")
        })?;

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
    fn match_prefix<'a, 'b>(&'a self, config_id: u8, word: &'b str) -> Option<(&'b str, &'a str)> {
        let fst = self.map.as_fst();
        let mut node = fst.root();

        if let Some(trans_idx) = node.find_input(config_id) {
            let t = node.transition(trans_idx);
            let mut current_output = t.out.value();
            node = fst.node(t.addr);

            let mut last_match: Option<(usize, u64)> = None;

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

                if let Ok(safe_index) = value_index.try_into()
                    && let Some(values) = metadata.values.get::<usize>(safe_index)
                    && let Some(first_value) = values.iter().next()
                {
                    let key = &word[..len];
                    return Some((key, first_value.as_str()));
                }
            }
        }

        None
    }
}
