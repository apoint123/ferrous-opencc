use std::path::PathBuf;

use clap::Parser;
use ferrous_opencc::FstDict;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,

    #[cfg(feature = "compress")]
    #[arg(long)]
    compress: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!(
        "Compiling {} -> {} ...",
        args.input.display(),
        args.output.display()
    );

    let dict = FstDict::from_text(&args.input)
        .map_err(|e| anyhow::anyhow!("Failed to load text dictionary: {e}"))?;

    dict.serialize_to_file(&args.output)
        .map_err(|e| anyhow::anyhow!("Failed to serialize binary dictionary: {e}"))?;

    #[cfg(feature = "compress")]
    if args.compress {
        let raw = std::fs::read(&args.output)?;
        let compressed = ferrous_opencc_compiler::compress_dictionary(&raw)
            .map_err(|e| anyhow::anyhow!("Failed to compress dictionary: {e}"))?;
        std::fs::write(&args.output, compressed)?;
    }

    println!("Compilation successful.");

    Ok(())
}
