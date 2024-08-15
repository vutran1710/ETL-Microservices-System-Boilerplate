mod mq;
mod server;

use clap::Parser;
use common::messages::Message;
use kanal::AsyncReceiver;
use mq::MessageQueue;
use server::Server;

#[cfg(feature = "example")]
use example::Etl;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    port: u16,
}

async fn main_task(receiver: AsyncReceiver<Message>) -> eyre::Result<()> {
    let etl = Etl::new();
    while let Ok(msg) = receiver.recv().await {
        etl.process(msg, msg_queue_sender).await?;
    }

    eyre::bail!("Message queue receiver exited unexpectedly")
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::try_init().ok();

    let Args { port } = Args::parse();

    let msg_queue = MessageQueue::new().await?;
    let server = Server::new(port);
    let (sender, receiver) = kanal::unbounded_async();

    tokio::try_join!(
        msg_queue.run(sender.clone()),
        main_task(receiver),
        server.run(sender.clone())
    )?;

    panic!("App exited unexpectedly")
}
