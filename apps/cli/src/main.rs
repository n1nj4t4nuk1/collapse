use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};
use collapse_core::{compress, extract, Algorithm};

#[derive(Parser)]
#[command(
    name = "collapse",
    about = "Compress and extract files from the command line."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compress a file.
    #[command(alias = "c")]
    Compress {
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
    },

    /// Extract an archive.
    #[command(alias = "e")]
    Extract {
        /// Archive to extract (.zip or .7z).
        file: PathBuf,

        /// Output directory. Defaults to current directory.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Compress {
            file,
            output,
            protocol,
            level,
        } => run_compress(&file, output, protocol, level),
        Command::Extract { file, output } => run_extract(&file, output),
    }
}

fn run_compress(file: &Path, output: Option<PathBuf>, protocol: Algorithm, level: u32) {
    if !file.exists() {
        eprintln!("error: file not found: {}", file.display());
        process::exit(1);
    }

    let output = output.unwrap_or_else(|| {
        let name = file.file_name().unwrap().to_string_lossy();
        PathBuf::from(format!("{}.{}", name, protocol.extension()))
    });

    let arcname = file.file_name().unwrap().to_string_lossy().to_string();

    match compress(file, &output, &arcname, protocol, level) {
        Ok(()) => println!("{}", output.display()),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}

fn run_extract(file: &Path, output: Option<PathBuf>) {
    if !file.exists() {
        eprintln!("error: file not found: {}", file.display());
        process::exit(1);
    }

    let output_dir = output.unwrap_or_else(|| PathBuf::from("."));

    match extract(file, &output_dir) {
        Ok(files) => {
            for f in &files {
                println!("{}", f);
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}
