use std::{
    collections::BTreeMap,
    fs::File,
    io::{
        BufRead,
        BufReader,
        Write,
    },
    path::Path,
};

use anyhow::{
    Context,
    Result,
};
use fst::MapBuilder;
use rkyv::{
    Archive,
    Deserialize,
    Serialize,
};

#[derive(Archive, Serialize, Deserialize, Debug)]
pub enum Delta {
    CharDiffs(Vec<(u16, char)>),
    FullReplacement(String),
}

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct SerializableFstDict {
    pub values: Vec<Vec<Delta>>,
    pub max_key_length: u32,
}

fn compute_delta(key: &str, value: &str) -> Delta {
    if key.chars().count() != value.chars().count() {
        return Delta::FullReplacement(value.to_string());
    }

    let diffs: Vec<(u16, char)> = key
        .chars()
        .zip(value.chars())
        .enumerate()
        .filter_map(|(i, (k, v))| if k != v { Some((i as u16, v)) } else { None })
        .collect();

    if diffs.len() * 6 > value.len() {
        return Delta::FullReplacement(value.to_string());
    }

    Delta::CharDiffs(diffs)
}

pub fn compile_dictionary(input_path: &Path) -> Result<Vec<u8>> {
    let file = File::open(input_path)
        .with_context(|| format!("Failed to open input dictionary: {}", input_path.display()))?;
    let reader = BufReader::new(file);

    let mut entries = BTreeMap::new();
    let mut max_key_length = 0u32;

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
                max_key_length = max_key_length.max(key.chars().count() as u32);
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

    let metadata = SerializableFstDict {
        values: values_vec,
        max_key_length,
    };

    let metadata_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&metadata)
        .map_err(|e| anyhow::anyhow!("Rkyv serialization failed: {e}"))?
        .into_vec();

    let mut final_bytes = Vec::new();

    final_bytes.write_all(&(metadata_bytes.len() as u64).to_le_bytes())?;
    final_bytes.write_all(&metadata_bytes)?;
    final_bytes.write_all(&fst_map_bytes)?;

    Ok(final_bytes)
}
