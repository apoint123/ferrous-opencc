use ferrous_opencc::{
    OpenCC,
    Result,
};

fn main() -> Result<()> {
    // Create OpenCC instance with built-in configuration
    let opencc = OpenCC::from_config(ferrous_opencc::config::BuiltinConfig::S2t)?;

    // Convert text
    let text = "开放中文转换是完全由 Rust 实现的。";
    let converted = opencc.convert(text);

    println!("{converted}");
    // Expected output: 開放中文轉換是完全由 Rust 實現的。

    assert_eq!(converted, "開放中文轉換是完全由 Rust 實現的。");
    Ok(())
}
