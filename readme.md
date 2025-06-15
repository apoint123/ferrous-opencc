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
cargo run --bin opencc-dict-compiler -- assets/dictionaries/STCharacters.txt
```

This will generate an `STCharacters.ocb` file in the same directory. The library will automatically use these `.ocb` files as a cache to speed up initial loading.

## License

This project is licensed under the [Apache-2.0 license](LICENSE).