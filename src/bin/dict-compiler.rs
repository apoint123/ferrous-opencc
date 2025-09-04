use clap::Parser;
use ferrous_opencc::dictionary::FstDict;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!(
        "Compiling {} -> {} ...",
        args.input.display(),
        args.output.display()
    );

    let dict = FstDict::from_text(&args.input)
        .map_err(|e| anyhow::anyhow!("Failed to load text dictionary: {}", e))?;

    dict.serialize_to_file(&args.output)
        .map_err(|e| anyhow::anyhow!("Failed to serialize binary dictionary: {}", e))?;

    println!("Compilation successful.");

    Ok(())
}
