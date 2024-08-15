mod mq;
mod server;

use clap::Parser;
use mq::MessageQueue;
use server::Server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    port: u16,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::try_init().ok();

    let Args { port } = Args::parse();

    let msg_queue = MessageQueue::new().await?;
    let server = Server::new(port);

    tokio::try_join!(msg_queue.run(), server.run())?;
    panic!("App exited unexpectedly")
}
