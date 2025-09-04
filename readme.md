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
ferrous-opencc = "0.2"
```

### Directory Structure

This library loads dictionaries and configuration files from the local filesystem. You can use the provided set of dictionary files, or compile your own and place them in the `assets/dictionaries/` folder.

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

You can obtain these dictionary and configuration files from the [official OpenCC repository](https://github.com/BYVoid/OpenCC/tree/master/data).

## Examples

### Method 1: Initialize with Configuration Name (Recommended)

Create an OpenCC instance using built-in configuration names, no external files required:

```rust
use ferrous_opencc::{OpenCC, Result};

fn main() -> Result<()> {
    // Create OpenCC instance with built-in configuration
    let opencc = OpenCC::from_config(ferrous_opencc::config::BuiltinConfig::S2t)?;

    // Convert text
    let text = "开放中文转换是完全由 Rust 实现的。";
    let converted = opencc.convert(text);

    println!("{}", converted);
    // Expected output: 開放中文轉換是完全由 Rust 實現的。

    assert_eq!(converted, "開放中文轉換是完全由 Rust 實現的。");
    Ok(())
}
```

**Supported Built-in Configuration Names:**
| Configuration Name     | Conversion Direction                                                            |
| :--------------------- | :------------------------------------------------------------------------------ |
| `BuiltinConfig::S2t`   | **Simplified → Traditional**                                                    |
| `BuiltinConfig::T2s`   | **Traditional → Simplified**                                                    |
| `BuiltinConfig::S2tw`  | Simplified → Traditional Chinese (Taiwan)                                       |
| `BuiltinConfig::Tw2s`  | Traditional Chinese (Taiwan) → Simplified                                       |
| `BuiltinConfig::S2hk`  | Simplified → Traditional Chinese (Hong Kong)                                    |
| `BuiltinConfig::Hk2s`  | Traditional Chinese (Hong Kong) → Simplified                                    |
| `BuiltinConfig::S2twp` | **Simplified → Traditional Chinese (Taiwan) (with Taiwan-specific vocabulary)** |
| `BuiltinConfig::Tw2sp` | **Traditional Chinese (Taiwan) (with Taiwan-specific vocabulary) → Simplified** |
| `BuiltinConfig::T2tw`  | Traditional → Traditional Chinese (Taiwan)                                      |
| `BuiltinConfig::Tw2t`  | Traditional Chinese (Taiwan) → Traditional                                      |
| `BuiltinConfig::T2hk`  | Traditional → Traditional Chinese (Hong Kong)                                   |
| `BuiltinConfig::Hk2t`  | Traditional Chinese (Hong Kong) → Traditional                                   |
| `BuiltinConfig::Jp2t`  | Japanese Shinjitai → Traditional                                                |
| `BuiltinConfig::T2jp`  | Traditional → Japanese Shinjitai                                                |

**Bold** entries indicate the most commonly used configurations.

### Method 2: Initialize with Configuration File

If you need to use custom configurations or external configuration files, here is a basic example of converting Simplified Chinese to Traditional Chinese:

```rust
use ferrous_opencc::{OpenCC, Result};

fn main() -> Result<()> {
    // Create an OpenCC instance with a specific configuration file.
    let opencc = OpenCC::new("assets/s2t.json")?;

    // Convert text.
    let text = "开放中文转换是完全由 Rust 实现的。";
    let converted = opencc.convert(text);

    println!("{}", converted);
    // Expected output: 開放中文轉換是完全由 Rust 實現的。

    assert_eq!(converted, "開放中文轉換是完全由 Rust 實現的。");
    Ok(())
}
```

## Command-Line Tool

This library provides a dictionary compilation tool that can compile text dictionaries into binary `.ocb` format.

You can run this binary target directly through Cargo.

```bash
cargo run --bin opencc-dict-compiler -- --input assets/dictionaries/STPhrases.txt --output ./STPhrases.ocb
```

This will generate an `STPhrases.ocb` file in the same directory. 

## Using Custom Dictionaries

While this library comes with all standard dictionaries embedded, you might need to load your own dictionary files in certain scenarios. For instance, you may have just compiled an `.ocb` file using the `opencc-dict-compiler` tool, or you might want to load dictionaries dynamically at runtime.

This requires you to create a conversion configuration manually, rather than relying on the built-in configurations.

### How It Works

1.  **Write a Custom Config File**: Create a `my_config.json` file to define your conversion pipeline. This config file must explicitly specify the paths to your dictionary files.
2.  **Create the Converter**: In your Rust code, directly create the `OpenCC` converter using the configuration file path.

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

#### 2. Use the Config in Rust

Now, you can write Rust code to use this configuration file.

```rust
use ferrous_opencc::{OpenCC, Result};

fn main() -> Result<()> {
    // Create a converter using the configuration file path
    let converter = OpenCC::new("my_config.json")?;

    // Perform the conversion
    let text = "我用路由器上网";
    let converted_text = converter.convert(text);
    
    println!("'{}' -> '{}'", text, converted_text);
    // Expected output: '我用路由器上网' -> '我用路由器上網'

    Ok(())
}
```

## License

This project is licensed under the [Apache-2.0 license](LICENSE).
