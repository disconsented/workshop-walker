use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use clap::Parser;

#[derive(Parser)]
#[command(about = "Create a SurrealDB migration stub file")]
struct Args {
    /// Name for the migration (appended after the timestamp)
    name: String,

    /// Directory to create the migration in
    #[arg(short, long, default_value = "migrations")]
    dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    let ts = Utc::now().timestamp_millis();
    let filename = format!("{}_{}.surql", ts, args.name);
    let path = args.dir.join(&filename);

    if let Err(e) = fs::create_dir_all(&args.dir) {
        eprintln!("error: could not create directory '{}': {e}", args.dir.display());
        std::process::exit(1);
    }

    if let Err(e) = fs::write(&path, "") {
        eprintln!("error: could not create '{}': {e}", path.display());
        std::process::exit(1);
    }

    println!("{}", path.display());
}
