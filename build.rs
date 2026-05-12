use std::{
    env,
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use anyhow::{
    Context,
    Result,
};
use ferrous_opencc_compiler::{
    CompilerChain,
    CompilerDictGroup,
    compile_global_dictionary,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    conversion_chain: Vec<ConversionNodeConfig>,
}

#[derive(Deserialize, Debug)]
struct ConversionNodeConfig {
    dict: DictConfig,
}

#[derive(Deserialize, Debug)]
struct DictConfig {
    #[serde(rename = "type")]
    dict_type: String,
    file: Option<String>,
    dicts: Option<Vec<Self>>,
}

fn extract_compiler_chain(chain_config: &[ConversionNodeConfig], dict_dir: &Path) -> CompilerChain {
    let mut groups = Vec::new();
    for node in chain_config {
        let mut paths = Vec::new();
        match node.dict.dict_type.as_str() {
            "group" => {
                if let Some(dicts) = &node.dict.dicts {
                    for d in dicts {
                        if let Some(f) = &d.file {
                            let actual_file = f.replace(".ocd2", ".txt");
                            paths.push(dict_dir.join(actual_file));
                        }
                    }
                }
            }
            "text" | "ocd2" => {
                if let Some(f) = &node.dict.file {
                    let actual_file = f.replace(".ocd2", ".txt");
                    paths.push(dict_dir.join(actual_file));
                }
            }
            _ => {}
        }
        groups.push(CompilerDictGroup { dict_paths: paths });
    }
    CompilerChain { groups }
}

fn run() -> Result<()> {
    let out_dir = env::var("OUT_DIR").context("Failed to get OUT_DIR")?;
    let dest_path = Path::new(&out_dir);
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").context("Failed to get CARGO_MANIFEST_DIR env variable")?,
    );

    let assets_root = manifest_dir.join("assets");
    let dict_dir = assets_root.join("dictionaries");

    let s2t = env::var("CARGO_FEATURE_S2T_CONVERSION").is_ok();
    let t2s = env::var("CARGO_FEATURE_T2S_CONVERSION").is_ok();
    let japanese = env::var("CARGO_FEATURE_JAPANESE_CONVERSION").is_ok();

    let mut all_configs: Vec<(u8, CompilerChain)> = Vec::new();

    if assets_root.exists() {
        for entry in fs::read_dir(&assets_root)? {
            let path = entry?.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file_stem = path.file_stem().unwrap().to_str().unwrap();

                let should_include = match file_stem {
                    "s2t" | "s2tw" | "s2hk" | "s2twp" | "tw2t" | "t2hk" => s2t,
                    "t2s" | "tw2s" | "hk2s" | "tw2sp" | "t2tw" | "hk2t" => t2s,
                    "jp2t" | "t2jp" => japanese,
                    _ => false,
                };

                if should_include {
                    let config_id: u8 = match file_stem {
                        "s2t" => 0,
                        "t2s" => 1,
                        "s2tw" => 2,
                        "tw2s" => 3,
                        "s2hk" => 4,
                        "hk2s" => 5,
                        "s2twp" => 6,
                        "tw2sp" => 7,
                        "t2tw" => 8,
                        "tw2t" => 9,
                        "t2hk" => 10,
                        "hk2t" => 11,
                        "jp2t" => 12,
                        "t2jp" => 13,
                        _ => continue,
                    };

                    let content = fs::read_to_string(&path)?;
                    let config: Config = serde_json::from_str(&content)?;
                    let compiler_chain =
                        extract_compiler_chain(&config.conversion_chain, &dict_dir);

                    all_configs.push((config_id, compiler_chain));
                }
            }
        }
    }

    if !all_configs.is_empty() {
        let ocb_bytes = compile_global_dictionary(&all_configs)?;

        #[cfg(feature = "compress")]
        let ocb_bytes = ferrous_opencc_compiler::compress_dictionary(&ocb_bytes)?;

        let ocb_path = dest_path.join("global_dictionary.ocb");
        fs::write(&ocb_path, &ocb_bytes)?;
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("cargo:warning=Build script ferrous-opencc/build.rs failed: {e:?}");
        std::process::exit(1);
    }
}
