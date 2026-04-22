use std::path::PathBuf;
use std::process;

use clap::Parser;
use collapse_core::{compress, Algorithm};

#[derive(Parser)]
#[command(
    name = "collapse",
    about = "Compress files from the command line."
)]
struct Cli {
    /// File to compress.
    file: PathBuf,

    /// Output archive path. Defaults to <file>.<extension>.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Compression algorithm: zip or 7z.
    #[arg(short, long, default_value = "zip")]
    protocol: Algorithm,

    /// Compression level (1 = fastest, 5 = max).
    #[arg(short, long, default_value_t = 5)]
    level: u32,
}

fn main() {
    let cli = Cli::parse();

    if !cli.file.exists() {
        eprintln!("error: file not found: {}", cli.file.display());
        process::exit(1);
    }

    let output = cli.output.unwrap_or_else(|| {
        let name = cli.file.file_name().unwrap().to_string_lossy();
        PathBuf::from(format!("{}.{}", name, cli.protocol.extension()))
    });

    let arcname = cli
        .file
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    match compress(&cli.file, &output, &arcname, cli.protocol, cli.level) {
        Ok(()) => println!("{}", output.display()),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}
