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
ferrous-opencc = "0.2"
```

### 目录结构

本库会从本地加载词典和配置文件。你可以使用本库附带的全套字典文件，或自行编译并放入 `assets/dictionaries/` 文件夹。

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

### 方法一：通过配置名初始化（推荐）

使用内置的配置名创建 OpenCC 实例，无需外部文件：

```rust
use ferrous_opencc::{OpenCC, Result};

fn main() -> Result<()> {
    // 使用内置配置创建 OpenCC 实例
    let opencc = OpenCC::from_config(ferrous_opencc::config::BuiltinConfig::S2t)?;

    // 转换文本
    let text = "开放中文转换是完全由 Rust 实现的。";
    let converted = opencc.convert(text);

    println!("{}", converted);
    // 预期输出: 開放中文轉換是完全由 Rust 實現的。

    assert_eq!(converted, "開放中文轉換是完全由 Rust 實現的。");
    Ok(())
}
```

**支持的内置配置名：**
| 配置名称               | 转换方向                              |
| :--------------------- | :------------------------------------ |
| `BuiltinConfig::S2t`   | **简体 → 繁体**                       |
| `BuiltinConfig::T2s`   | **繁体 → 简体**                       |
| `BuiltinConfig::S2tw`  | 简体 → 台湾正体                       |
| `BuiltinConfig::Tw2s`  | 台湾正体 → 简体                       |
| `BuiltinConfig::S2hk`  | 简体 → 香港繁体                       |
| `BuiltinConfig::Hk2s`  | 香港繁体 → 简体                       |
| `BuiltinConfig::S2twp` | **简体 → 台湾正体（含台湾特定词汇）** |
| `BuiltinConfig::Tw2sp` | **台湾正体（含台湾特定词汇）→ 简体**  |
| `BuiltinConfig::T2tw`  | 繁体 → 台湾正体                       |
| `BuiltinConfig::Tw2t`  | 台湾正体 → 繁体                       |
| `BuiltinConfig::T2hk`  | 繁体 → 香港繁体                       |
| `BuiltinConfig::Hk2t`  | 香港繁体 → 繁体                       |
| `BuiltinConfig::Jp2t`  | 日本新字体 → 繁体                     |
| `BuiltinConfig::T2jp`  | 繁体 → 日本新字体                     |

**加粗**的条目为最常用的配置。

### 方法二：通过配置文件初始化

如果你需要使用自定义配置或外部配置文件，这是一个将简体中文转换为繁体中文的基础示例

```rust
use ferrous_opencc::{OpenCC, Result};

fn main() -> Result<()> {
    // 使用指定的配置文件创建一个 OpenCC 实例。
    let opencc = OpenCC::new("assets/s2t.json")?;

    // 转换文本。
    let text = "开放中文转换是完全由 Rust 实现的。";
    let converted = opencc.convert(text);

    println!("{}", converted);
    // 预期输出: 開放中文轉換是完全由 Rust 實現的。

    assert_eq!(converted, "開放中文轉換是完全由 Rust 實現的。");
    Ok(())
}
```

## 命令行工具

本库提供了一个词典编译工具，可以将文本词典编译成二进制的 `.ocb` 格式。

你可以通过 Cargo 直接运行这个二进制目标。

```bash
cargo run --bin opencc-dict-compiler -- --input assets/dictionaries/STPhrases.txt --output ./STPhrases.ocb
```

这会在相同目录下生成 `STPhrases.ocb` 文件。

## 使用自定义词典

虽然本库内置了所有标准词典，但在某些场景下，你可能需要加载自己的词典文件。例如，你可能刚刚使用 `opencc-dict-compiler` 工具编译了一个 `.ocb` 文件，或者你想在运行时动态加载词典。

这需要你手动创建转换配置，而不是依赖内置的配置。

### 实现步骤

1.  **编写自定义配置文件**: 创建一个 `my_config.json` 文件来定义你的转换流程。这个配置文件需要明确指定词典文件的路径。
2.  **创建转换器**: 在你的 Rust 代码中，直接使用配置文件路径来创建 `OpenCC` 转换器。

### 示例

假设你已经通过编译工具生成了 `my_dicts/my_s2t_phrases.ocb` 和 `my_dicts/my_s2t_chars.ocb` 这两个自定义词典。

#### 1. 创建 `my_config.json`

在你的项目根目录下创建一个名为 `my_config.json` 的文件，内容如下：

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

**注意**:
- 使用 `"type": "ocd2"` 来告诉库这是一个二进制词典文件。虽然扩展名是 `.ocb`，但它的格式与 OpenCC v2 的 `.ocd2` 兼容。
- `file` 字段中的路径是**相对于你的可执行文件运行时的当前工作目录**。

#### 2. 在 Rust 代码中使用配置

现在，你可以编写 Rust 代码来使用这个配置文件。

```rust
use ferrous_opencc::{OpenCC, Result};

fn main() -> Result<()> {
    // 使用配置文件路径创建转换器
    let converter = OpenCC::new("my_config.json")?;

    // 执行转换
    let text = "我用路由器上网";
    let converted_text = converter.convert(text);
    
    println!("'{}' -> '{}'", text, converted_text);
    // 预期输出: '我用路由器上网' -> '我用路由器上網'

    Ok(())
}
```

## 开源协议

本项目使用 [Apache-2.0 license](LICENSE) 开源协议。
