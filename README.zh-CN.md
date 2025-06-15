# Ferrous-opencc

[![CI Status](https://github.com/apoint123/ferrous-opencc/actions/workflows/ci.yml/badge.svg)](https://github.com/apoint123/ferrous-opencc/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ferrous-opencc.svg)](https://crates.io/crates/ferrous-opencc)
[![Docs.rs](https://docs.rs/ferrous-opencc/badge.svg)](https://docs.rs/ferrous-opencc)

一个纯 Rust 实现的 [OpenCC](https://github.com/BYVoid/OpenCC) 项目，致力于为繁体中文和简体中文之间提供高性能、高可靠性的转换。

[English README](README.md)

## 特性

-   **高性能**: 使用 `FST` (有限状态转换器) 进行高效的词典查询，性能远超于哈希表的实现。
-   **纯 Rust**: 不依赖任何 C++ 库。完全 Rust 实现。
-   **易于扩展**: 支持加载自定义的 OpenCC 配置文件和词典。
-   **工具链完备**: 自带一个命令行工具，可将文本词典编译为高效的 `.ocb` 二进制格式。

## 快速开始

将 `ferrous-opencc` 添加到你的 `Cargo.toml` 中：

```toml
[dependencies]
ferrous-opencc = "*"
```

### 目录结构

本库会从本地加载词典和配置文件。你可以使用我准备好的全套字典文件，或自行编译并放入 `assets/dictionaries/` 文件夹。

```
你的项目/
├── assets/
│   ├── dictionaries/
│   │   ├── STPhrases.txt
│   │   ├── STCharacters.txt
│   │   ├── TPhrases.txt
│   │   └── ... (其他 .txt 词典文件)
│   └── s2t.json
└── src/
    └── main.rs
```

可以从 [OpenCC 官方仓库](https://github.com/BYVoid/OpenCC/tree/master/data) 获取这些词典和配置文件。

## 示例

一个将简体中文转换为繁体中文的基础示例

```rust
use ferrous_opencc::{OpenCC, Result};

fn main() -> Result<()> {
    // 使用指定的配置文件创建一个 OpenCC 实例。
    let opencc = OpenCC::new("assets/dictionaries/s2t.json")?;

    // 转换文本。
    let text = "“开放中文转换”是完全由 Rust 实现的。";
    let converted = opencc.convert(text);

    println!("{}", converted);
    // 预期输出: 「開放中文轉換」是完全由 Rust 實現的。

    assert_eq!(converted, "「開放中文轉換」是完全由 Rust 實現的。");
    Ok(())
}
```

## 命令行工具

本库提供了一个词典编译工具，可以将文本词典编译成二进制的 `.ocb` 格式。

你可以通过 Cargo 直接运行这个二进制目标。

```bash
cargo run --bin opencc-dict-compiler -- assets/dictionaries/STCharacters.txt
```

这会在相同目录下生成 `STCharacters.ocb` 文件。程序库会自动将这些 `.ocb` 文件作为缓存使用，从而加速程序的初次加载。

## 开源协议

本项目使用 [Apache-2.0 license](LICENSE) 开源协议。
