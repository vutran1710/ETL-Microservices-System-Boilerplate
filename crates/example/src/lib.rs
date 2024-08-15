use clap::Parser;
// use database::tier_1::Post;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source_db: String,
}

pub fn main() -> eyre::Result<()> {
    let Args { source_db } = Args::parse();
    let _conn = database::connection(&source_db);
    Ok(())
}
