#[derive(Encode, Decode, Debug)]
pub struct SerializableFstDict {
    pub values: Vec<Vec<Arc<str>>>,
    pub max_key_length: usize,
}

pub fn compile_dictionary(input_path: &Path) -> Result<Vec<u8>> {
    let file = File::open(input_path)
        .with_context(|| format!("Failed to open input dictionary: {}", input_path.display()))?;
    let reader = BufReader::new(file);

    let mut entries = BTreeMap::new();
    let mut max_key_length = 0;

    for line in reader.lines() {
        let line = line.with_context(|| "Failed to read line from dictionary")?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() == 2 {
            let key = parts[0];
            let values: Vec<Arc<str>> = parts[1].split(' ').map(|s| s.into()).collect();

            if !key.is_empty() && !values.is_empty() && !values.iter().any(|s| s.is_empty()) {
                max_key_length = max_key_length.max(key.chars().count());
                entries.insert(key.to_string(), values);
            }
        }
    }

    let mut values_vec = Vec::with_capacity(entries.len());
    let mut builder = MapBuilder::memory();

    for (key, values) in entries {
        let index = values_vec.len() as u64;
        values_vec.push(values);
        builder.insert(key, index).with_context(|| "Failed to insert key-value pair into FST")?;
    }

    let fst_map_bytes = builder
        .into_inner()
        .with_context(|| "Failed to finalize FST construction")?;

    let metadata = SerializableFstDict {
        values: values_vec,
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
