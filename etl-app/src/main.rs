mod mq;
mod server;

use clap::Parser;
use common::messages::Message;
use kanal::AsyncReceiver;
use mq::MessageQueue;
use server::Server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    port: u16,
}

async fn main_task(receiver: AsyncReceiver<Message>) -> eyre::Result<()> {
    while let Ok(msg) = receiver.recv().await {
        match msg {
            Message::ProcessAll => {
                log::info!("Received message to process all");
                todo!("Calling crate function to process all")
            }
            Message::ProcessOne { id } => {
                log::info!("Received message to process one with id: {}", id);
                todo!("Calling crate function to process one")
            }
            Message::ProcessIds { ids } => {
                log::info!("Received message to process list of ids: {:?}", ids);
                todo!("Calling crate function to process ids")
            }
            Message::ProcessRange { from, to } => {
                log::info!(
                    "Received message to process range from {:?} to {:?}",
                    from,
                    to
                );
                todo!("Calling crate function to process range")
            }
        }
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
