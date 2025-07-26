//! 实现 `FstDict`，一个基于 FST 的高性能词典。
//! 优先从预编译的 `.ocb` 文件加载，
//! 如果找不到，则从 `.txt` 文件编译。

use crate::dictionary::Dictionary;
use crate::error::Result;
use bincode::{Decode, Encode, config};
use fst::Map;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::sync::Arc;

/// 一个使用 FST 实现的词典。
/// 包含用于快速查询的 FST 映射、存储实际字符串值的向量，
/// 以及用于优化的最长键长度。
#[derive(Debug)]
pub struct FstDict {
    /// FST 映射，将键映射到 `values` 向量中的索引
    map: Map<Vec<u8>>,
    /// 包含词典中所有不重复的值的向量
    values: Vec<Vec<Arc<str>>>,
    /// 词典中最长键的长度
    max_key_length: usize,
}

/// 用于序列化词典中非 FST 部分的辅助结构体
#[derive(Encode, Decode)]
struct SerializableFstDict {
    values: Vec<Vec<Arc<str>>>,
    max_key_length: usize,
}

impl FstDict {
    /// 从给定路径创建一个新的 `FstDict` 实例。
    /// 先从预编译的 `.ocb` 加载，
    /// 没有再从文本文件编译
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let compiled_path = path.with_extension("ocb");

        // 检查是否存在预编译的文件
        if compiled_path.is_file() {
            // 如果文本源文件也存在，则检查修改时间，判断缓存是否有效
            if let Ok(text_meta) = path.metadata() {
                if let Ok(compiled_meta) = compiled_path.metadata() {
                    let text_modified = text_meta.modified()?;
                    let compiled_modified = compiled_meta.modified()?;
                    if compiled_modified > text_modified {
                        // 缓存比源文件新，可以使用缓存
                        return Self::from_ocb_file(&compiled_path);
                    }
                }
            } else {
                // 源文件不存在，但缓存存在，直接使用缓存
                return Self::from_ocb_file(&compiled_path);
            }
        }

        // 无法使用缓存，则从文本文件编译，并创建新的缓存
        let dict = Self::from_text(path)?;
        // 序列化失败也无所谓，只是意味着下次没有缓存用。可以忽略
        let _ = dict.serialize_to_file(&compiled_path);
        Ok(dict)
    }

    /// 从 `.ocb` 加载词典
    fn from_ocb_file(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }

    /// 序列化词典
    pub fn serialize_to_file(&self, path: &Path) -> Result<()> {
        let metadata = SerializableFstDict {
            values: self.values.clone(),
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

    /// 从 OpenCC 格式的文本文件创建词典
    pub fn from_text(path: &Path) -> Result<Self> {
        let ocb_bytes = crate::compiler::compile_dictionary(path)?;
        Self::from_ocb_bytes(&ocb_bytes)
    }

    /// 从内存中的 `.ocb` 字节数组创建词典
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

        let mut fst_bytes = Vec::new();
        reader.read_to_end(&mut fst_bytes)?;

        let map = Map::new(fst_bytes)?;

        Ok(Self {
            map,
            values: metadata.values,
            max_key_length: metadata.max_key_length,
        })
    }
}

impl Dictionary for FstDict {
    /// 查找输入字符串在词典中的最长前缀匹配
    fn match_prefix<'a, 'b>(&'a self, word: &'b str) -> Option<(&'b str, &'a [Arc<str>])> {
        let fst = self.map.as_fst();
        let mut node = fst.root();

        // `last_match` 用于记录到目前为止找到的最长匹配的 (字节长度, 最终值的索引)
        let mut last_match: Option<(usize, u64)> = None;

        // `current_output` 用于累加路径上的输出值
        let mut current_output: u64 = 0;

        // 逐字节遍历输入字符串
        for (i, byte) in word.as_bytes().iter().enumerate() {
            // 在当前节点的所有转换中，查找与当前字节匹配的转换
            if let Some(trans_idx) = node.find_input(*byte) {
                let t = node.transition(trans_idx);
                // 累加当前转换的输出值
                current_output += t.out.value();
                // 移动到转换指向的下一个节点
                node = fst.node(t.addr);

                // 如果新状态是一个终点，说明匹配到了一个完整的词
                if node.is_final() {
                    // 完整的值 = 路径累加值 + 终点额外值
                    let final_value = current_output + node.final_output().value();
                    // 记录下这个匹配。因为我们从左到右遍历，这个记录会被更长的匹配覆盖。
                    last_match = Some((i + 1, final_value));
                }
            } else {
                // 当前字节没有对应的转换，说明不可能有更长的前缀了，退出循环
                break;
            }
        }

        // 循环结束后，`last_match` 中就保存了最长的前缀匹配信息
        if let Some((len, value_index)) = last_match {
            // 使用计算出的索引来获取值
            if let Some(values) = self.values.get(value_index as usize) {
                let key = &word[..len];
                return Some((key, values.as_slice()));
            }
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

    use super::*;
    use tempfile::tempdir;

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
        let values_str: Vec<&str> = values.iter().map(|v| v.as_ref()).collect();
        assert_eq!(values_str, ["一個"]);
        let (key, values) = dict.match_prefix("一个半小时").unwrap();
        assert_eq!(key, "一个半");
        let values_str: Vec<&str> = values.iter().map(|v| v.as_ref()).collect();
        assert_eq!(values_str, ["一個半"]);
        let values_str: Vec<&str> = values.iter().map(|v| v.as_ref()).collect();
        assert_eq!(values_str, ["一個半"]);

        let (key, values) = dict.match_prefix("世纪之交").unwrap();
        assert_eq!(key, "世纪");
        let values_str: Vec<&str> = values.iter().map(|v| v.as_ref()).collect();
        assert_eq!(values_str, ["世紀"]);
        let (key, values) = dict.match_prefix("一").unwrap();
        assert_eq!(key, "一");
        let values_str: Vec<&str> = values.iter().map(|v| v.as_ref()).collect();
        assert_eq!(values_str, ["一"]);
        let (key, values) = dict.match_prefix("一").unwrap();
        assert_eq!(key, "一");
        let values_str: Vec<&str> = values.iter().map(|v| v.as_ref()).collect();
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
        let values_str: Vec<&str> = values.iter().map(|v| v.as_ref()).collect();
        assert_eq!(values_str, ["Hello"]);

        let (key, values) = dict_from_ocb.match_prefix("你好世界").unwrap();
        assert_eq!(key, "你好");
        let values_str: Vec<&str> = values.iter().map(|v| v.as_ref()).collect();
        assert_eq!(values_str, ["Hello"]);
    }
}
