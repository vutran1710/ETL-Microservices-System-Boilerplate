use clap::Parser;
use database::connection;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source_db: String,
}

pub fn etl() -> eyre::Result<()> {
    let Args { source_db } = Args::parse();
    let _conn = connection(&source_db);
    Ok(())
}
