use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    sync::Arc,
};

use anyhow::{Context, Result};
use bincode::{Decode, Encode, config};
use fst::MapBuilder;

#[derive(Encode, Decode, Debug)]
pub struct SerializableOptimizedValues {
    pub string_pool: Vec<Arc<str>>,
    pub flat_indices: Vec<u32>,
    pub offsets: Vec<u32>,
}

#[derive(Encode, Decode, Debug)]
pub enum Delta {
    /// 存储字符级别的差异 (索引, 新字符)
    CharDiffs(Vec<(u16, char)>),
    /// 差异过大时，直接存储完整的新字符串
    FullReplacement(Arc<str>),
}

#[derive(Encode, Decode, Debug)]
pub struct SerializableFstDict {
    pub compressed_values: Vec<u8>,
    pub max_key_length: usize,
}

fn compute_delta(key: &str, value: &str) -> Delta {
    if key.chars().count() != value.chars().count() {
        return Delta::FullReplacement(value.into());
    }

    let diffs: Vec<(u16, char)> = key
        .chars()
        .zip(value.chars())
        .enumerate()
        .filter_map(|(i, (k, v))| if k != v { Some((i as u16, v)) } else { None })
        .collect();

    if diffs.len() * 6 > value.len() {
        return Delta::FullReplacement(value.into());
    }

    Delta::CharDiffs(diffs)
}

pub fn compile_dictionary(input_path: &Path) -> Result<Vec<u8>> {
    let file = File::open(input_path)
        .with_context(|| format!("Failed to open input dictionary: {}", input_path.display()))?;
    let reader = BufReader::new(file);

    let mut entries = BTreeMap::new();
    let mut max_key_length = 0;

    for line in reader.lines() {
        let line = line.with_context(|| "Failed to read line from dictionary")?;

        let trimmed_line = line.trim();
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();

        if parts.len() == 2 {
            let key = parts[0];
            let values: Vec<&str> = parts[1].split(' ').collect();

            if !key.is_empty() && !values.is_empty() && !values.iter().any(|s| s.is_empty()) {
                max_key_length = max_key_length.max(key.chars().count());
                let delta_values = values.into_iter().map(|v| compute_delta(key, v)).collect();
                entries.insert(key.to_string(), delta_values);
            }
        }
    }

    let mut values_vec: Vec<Vec<Delta>> = Vec::with_capacity(entries.len());
    let mut builder = MapBuilder::memory();

    for (key, values) in entries {
        let index = values_vec.len() as u64;
        values_vec.push(values);
        builder
            .insert(key, index)
            .with_context(|| "Failed to insert key-value pair into FST")?;
    }

    let fst_map_bytes = builder
        .into_inner()
        .with_context(|| "Failed to finalize FST construction")?;

    let values_bytes = bincode::encode_to_vec(&values_vec, config::standard())
        .with_context(|| "Bincode values serialization failed")?;

    let compressed_values =
        zstd::encode_all(&values_bytes[..], 0).with_context(|| "Zstd compression failed")?;

    let metadata = SerializableFstDict {
        compressed_values,
        max_key_length,
    };

    let metadata_bytes = bincode::encode_to_vec(&metadata, config::standard())
        .with_context(|| "Bincode metadata serialization failed")?;

    let mut final_bytes = Vec::new();

    final_bytes.write_all(&(metadata_bytes.len() as u64).to_le_bytes())?;
    final_bytes.write_all(&metadata_bytes)?;
    final_bytes.write_all(&fst_map_bytes)?;

    Ok(final_bytes)
}
