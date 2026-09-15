use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    xml: PathBuf,
    out: PathBuf,
}

fn main() -> Result<(), bf_viewer_core::Error> {
    let cli = Cli::parse();
    let index = bf_viewer_core::create_index(&cli.xml)?;
    bf_viewer_core::write_index(&index, &cli.out)?;
    println!(
        "{} records saved to index at {}",
        index.len(),
        cli.out.to_str().unwrap()
    );
    Ok(())
}
