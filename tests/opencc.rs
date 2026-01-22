use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::PathBuf,
};

use ferrous_opencc::{
    OpenCC,
    config::BuiltinConfig,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct TestSuite {
    cases: Vec<TestCase>,
}

#[derive(Deserialize, Debug)]
struct TestCase {
    id: String,
    input: String,
    // 配置文件名, 预期输出
    expected: HashMap<String, String>,
}

#[test]
fn test_official_compatibility() {
    let mut json_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    json_path.push("tests");
    json_path.push("data");
    json_path.push("testcases.json");

    assert!(
        json_path.exists(),
        "testcases.json not found at {}",
        json_path.display()
    );

    let file = File::open(&json_path).expect("Failed to open testcases.json");
    let reader = BufReader::new(file);
    let suite: TestSuite = serde_json::from_reader(reader).expect("Failed to parse testcases.json");

    let mut converters: HashMap<String, OpenCC> = HashMap::new();

    for case in suite.cases {
        for (config_key, expected_output) in case.expected {
            if !converters.contains_key(&config_key) {
                let filename = format!("{config_key}.json");

                match BuiltinConfig::from_filename(&filename) {
                    Ok(builtin_config) => {
                        let opencc = OpenCC::from_config(builtin_config)
                            .unwrap_or_else(|_| panic!("Failed to load config: {filename}"));
                        converters.insert(config_key.clone(), opencc);
                    }
                    Err(e) => {
                        panic!("load config failed, check if you enabled all features: {e:?}");
                    }
                }
            }

            if let Some(opencc) = converters.get(&config_key) {
                let actual = opencc.convert(&case.input);

                assert_eq!(
                    actual, expected_output,
                    "\nTest Failed!\nCase ID: {}\nConfig: {}\nInput:   {}\nExpected: {}\nActual:   {}\n",
                    case.id, config_key, case.input, expected_output, actual
                );
            }
        }
    }
}
