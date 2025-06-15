# Ferrous-opencc

[![CI Status](https://github.com/apoint123/ferrous-opencc/actions/workflows/ci.yml/badge.svg)](https://github.com/apoint123/ferrous-opencc/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ferrous-opencc.svg)](https://crates.io/crates/ferrous-opencc)
[![Docs.rs](https://docs.rs/ferrous-opencc/badge.svg)](https://docs.rs/ferrous-opencc)

A pure Rust implementation of the [OpenCC](https://github.com/BYVoid/OpenCC) project, dedicated to providing high-performance and reliable conversion between Traditional and Simplified Chinese.

[中文 README](README.zh-CN.md)

## Features

-   **High-Performance**: Utilizes `FST` (Finite State Transducers) for efficient dictionary lookups, significantly outperforming HashMap-based implementations.
-   **Pure Rust**: No C++ dependencies. Implemented entirely in Rust.
-   **Extensible**: Supports loading custom OpenCC configuration files and dictionaries.
-   **Comprehensive Tooling**: Includes a command-line tool to compile text dictionaries into an efficient `.ocb` binary format.

## Quick Start

Add `ferrous-opencc` to your `Cargo.toml`:

```toml
[dependencies]
ferrous-opencc = "*"
```

### Directory Structure

This library loads dictionaries and configuration files from the local filesystem. You can use the complete set of dictionary files I've prepared, or compile your own and place them in the `assets/dictionaries/` folder.

```
your-project/
├── assets/
│   ├── dictionaries/
│   │   ├── STPhrases.txt
│   │   ├── STCharacters.txt
│   │   ├── TPhrases.txt
│   │   └── ... (other .txt dictionary files)
│   └── s2t.json
└── src/
    └── main.rs
```

You can obtain these dictionary and configuration files from the [official OpenCC repository](https://github.com/BYVoid/OpenCC).

## Example

A basic example of converting Simplified Chinese to Traditional Chinese.

```rust
use ferrous_opencc::{OpenCC, Result};

fn main() -> Result<()> {
    // Create an OpenCC instance with a specific configuration file.
    let opencc = OpenCC::new("assets/s2t.json")?;

    // Convert text.
    let text = "“开放中文转换”是完全由 Rust 实现的。";
    let converted = opencc.convert(text);

    println!("{}", converted);
    // Expected output: 「開放中文轉換」是完全由 Rust 實現的。

    assert_eq!(converted, "「開放中文轉換」是完全由 Rust 實現的。");
    Ok(())
}
```

## Command-Line Tool

This library provides a dictionary compilation tool that can compile text dictionaries into binary `.ocb` format.

You can run this binary target directly through Cargo.

```bash
cargo run --bin opencc-dict-compiler -- --input assets/dictionaries/STPhrases.txt --output ./STPhrases.ocb
```

This will generate an `STCharacters.ocb` file in the same directory. 

## Using Custom Dictionaries

While this library comes with all standard dictionaries embedded, you might need to load your own dictionary files in certain scenarios. For instance, you may have just compiled an `.ocb` file using the `opencc-dict-compiler` tool, or you might want to load dictionaries dynamically at runtime.

This requires you to create a conversion configuration manually, rather than relying on the built-in configurations.

### How It Works

1.  **Write a Custom Config File**: Create a `my_config.json` file to define your conversion pipeline. This config file must explicitly specify the paths to your dictionary files.
2.  **Load the Config File**: In your Rust code, use `ferrous_opencc::Config` to load this JSON file.
3.  **Create the Converter**: Instantiate the `OpenCC` converter using the loaded `Config` object.

### Example

Let's assume you have generated `my_dicts/my_s2t_phrases.ocb` and `my_dicts/my_s2t_chars.ocb` using the compiler tool.

#### 1. Create `my_config.json`

Create a file named `my_config.json` in your project's root directory with the following content:

```json
{
  "name": "My-Simplified-to-Traditional-Conversion",
  "segmentation": {
    "type": "mm",
    "dict": {
      "type": "ocd2",
      "file": "my_dicts/my_s2t_phrases.ocb"
    }
  },
  "conversion_chain": [
    {
      "dict": {
        "type": "ocd2",
        "file": "my_dicts/my_s2t_phrases.ocb"
      }
    },
    {
      "dict": {
        "type": "ocd2",
        "file": "my_dicts/my_s2t_chars.ocb"
      }
    }
  ]
}
```
**Note**:
- Use `"type": "ocd2"` to inform the library that this is a binary dictionary file. Although our extension is `.ocb`, its format is compatible with OpenCC v2's `.ocd2`.
- The path in the `file` field is **relative to the current working directory** where your executable is run.

#### 2. Load the Config in Rust

Now, you can write Rust code to load and use this configuration file.

```rust
use ferrous_opencc::{Config, OpenCC};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // 1. Load the custom configuration from a file
    let config_path = Path::new("my_config.json");
    let config = Config::from_file(config_path)?;

    // 2. Create a converter using the loaded config
    let converter = OpenCC::from_config(config)?;

    // 3. Perform the conversion
    let text = "我用路由器上网";
    let converted_text = converter.convert(text);
    
    println!("'{}' -> '{}'", text, converted_text);
    // Expected output: '我用路由器上网' -> '我用路由器上網'

    Ok(())
}
```

## License

This project is licensed under the [Apache-2.0 license](LICENSE).