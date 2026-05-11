use std::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
    fs::File,
    io::{
        BufRead,
        BufReader,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
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
pub struct SerializableFstDict {
    pub values: Vec<Vec<String>>,
    pub max_key_length: u32,
}

pub struct CompilerDictGroup {
    pub dict_paths: Vec<PathBuf>,
}

pub struct CompilerChain {
    pub groups: Vec<CompilerDictGroup>,
}

fn load_txt_dict(path: &Path) -> Result<BTreeMap<String, Vec<String>>> {
    let file = File::open(path).with_context(|| format!("Failed to open: {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut entries: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for line in reader.lines() {
        let line = line.with_context(|| "Failed to read line")?;

        let trimmed_line = line.trim();
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        if let Some((key, values_str)) = trimmed_line.split_once('\t') {
            if key.is_empty() {
                continue;
            }

            let values: Vec<String> = values_str
                .split_ascii_whitespace()
                .map(String::from)
                .collect();

            if !values.is_empty() {
                entries.entry(key.to_string()).or_default().extend(values);
            }
        }
    }
    Ok(entries)
}

fn build_fst_from_map(
    entries: BTreeMap<String, Vec<String>>,
    max_key_length: u32,
) -> Result<Vec<u8>> {
    let mut values_vec: Vec<Vec<String>> = Vec::with_capacity(entries.len());
    let mut builder = MapBuilder::memory();

    for (key, values) in entries {
        let index = values_vec.len() as u64;
        values_vec.push(values);
        builder
            .insert(key, index)
            .with_context(|| "Failed to insert into FST")?;
    }

    let fst_map_bytes = builder
        .into_inner()
        .with_context(|| "Failed to finalize FST")?;

    let metadata = SerializableFstDict {
        values: values_vec,
        max_key_length,
    };

    let metadata_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&metadata)
        .map_err(|e| anyhow::anyhow!("Rkyv serialization failed: {e}"))?
        .into_vec();

    let total_size = 8 + metadata_bytes.len() + fst_map_bytes.len();
    let mut final_bytes = Vec::with_capacity(total_size);

    final_bytes.write_all(&(metadata_bytes.len() as u64).to_le_bytes())?;
    final_bytes.write_all(&metadata_bytes)?;
    final_bytes.write_all(&fst_map_bytes)?;

    Ok(final_bytes)
}

fn convert_str_with_map(text: &str, map: &BTreeMap<String, Vec<String>>, max_len: usize) -> String {
    let mut result = String::with_capacity(text.len());
    let mut i = 0;

    while i < text.len() {
        let remaining = &text[i..];
        let mut matched = false;

        let check_len = max_len.min(remaining.len());
        for len in (1..=check_len).rev() {
            if remaining.is_char_boundary(len) {
                let prefix = &remaining[..len];
                if let Some(values) = map.get(prefix) {
                    result.push_str(&values[0]);
                    i += len;
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            let ch = remaining.chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }
    result
}

fn generate_reverse_keys(
    target: &str,
    reverse_map: &BTreeMap<String, Vec<String>>,
    rev_max_len: usize,
    current_path: &mut String,
    results: &mut BTreeSet<String>,
) {
    if target.is_empty() {
        results.insert(current_path.clone());
        return;
    }

    let check_len = rev_max_len.min(target.len());

    for len in (1..=check_len).rev() {
        if target.is_char_boundary(len) {
            let prefix = &target[..len];
            let remainder = &target[len..];

            if let Some(original_keys) = reverse_map.get(prefix) {
                for orig_k in original_keys {
                    let old_len = current_path.len();
                    current_path.push_str(orig_k);

                    generate_reverse_keys(
                        remainder,
                        reverse_map,
                        rev_max_len,
                        current_path,
                        results,
                    );

                    current_path.truncate(old_len);
                }
            }
        }
    }

    let ch = target.chars().next().unwrap();
    let remainder = &target[ch.len_utf8()..];

    let old_len = current_path.len();
    current_path.push(ch);
    generate_reverse_keys(remainder, reverse_map, rev_max_len, current_path, results);
    current_path.truncate(old_len);
}

pub fn compile_chain(chain: &CompilerChain) -> Result<Vec<u8>> {
    let mut flat_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (i, group) in chain.groups.iter().enumerate() {
        let mut group_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut group_max_len = 0;

        for dict_path in &group.dict_paths {
            let dict_entries = load_txt_dict(dict_path)?;
            for (k, v) in dict_entries {
                group_max_len = group_max_len.max(k.len());
                group_map.entry(k).or_default().extend(v);
            }
        }

        if i == 0 {
            flat_map = group_map;
        } else {
            let mut next_flat_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

            let mut reverse_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let mut rev_max_len = 0;
            for (k, values) in flat_map.iter() {
                if let Some(v) = values.first() {
                    reverse_map.entry(v.clone()).or_default().push(k.clone());
                    rev_max_len = rev_max_len.max(v.len());
                }
            }

            for (k, values) in flat_map.into_iter() {
                let mut new_values = Vec::with_capacity(values.len());
                for v in values {
                    let converted_v = convert_str_with_map(&v, &group_map, group_max_len);
                    new_values.push(converted_v);
                }
                next_flat_map.insert(k, new_values);
            }

            for (group_k, group_v) in group_map.into_iter() {
                next_flat_map
                    .entry(group_k.clone())
                    .or_insert_with(|| group_v.clone());

                let mut results = BTreeSet::new();
                let mut current_path = String::with_capacity(&group_k.len() * 2);
                generate_reverse_keys(
                    &group_k,
                    &reverse_map,
                    rev_max_len,
                    &mut current_path,
                    &mut results,
                );

                for rev_k in results {
                    next_flat_map
                        .entry(rev_k)
                        .or_insert_with(|| group_v.clone());
                }
            }

            flat_map = next_flat_map;
        }
    }

    let max_len_usize = flat_map.keys().map(|k| k.len()).max().unwrap_or_default();
    let final_max_length = max_len_usize
        .try_into()
        .expect("Dictionary key length exceeds u32::MAX (4GB)");

    build_fst_from_map(flat_map, final_max_length)
}
