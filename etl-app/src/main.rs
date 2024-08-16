mod mq;
mod server;

use clap::Parser;
use common::messages::Message;
use common::ETLTrait;

use kanal::AsyncReceiver;
use kanal::AsyncSender;
use mq::MessageQueue;
use server::Server;

#[cfg(feature = "example")]
use example::Etl;
#[cfg(feature = "example")]
use example::SinkOrderingID;
#[cfg(feature = "example")]
use example::SourceOrderingID;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source: String,

    #[arg(short, long)]
    sink: String,

    #[arg(short, long)]
    port: u16,
}

async fn main_task(
    source: &str,
    sink: &str,
    receiver: AsyncReceiver<Message<SourceOrderingID>>,
    emitter: AsyncSender<Message<SinkOrderingID>>,
) -> eyre::Result<()> {
    let etl = Etl::new(source, sink).await?;

    while let Ok(msg) = receiver.recv().await {
        etl.process_message(msg, emitter.clone()).await?;
    }

    eyre::bail!("Message queue receiver exited unexpectedly")
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::try_init().ok();

    let Args { port, source, sink } = Args::parse();

    let msg_queue = MessageQueue::new().await?;
    let server = Server::new(port);

    let (input_sender, input_receiver) = kanal::unbounded_async();
    let (output_sender, output_receiver) = kanal::unbounded_async();

    tokio::try_join!(
        msg_queue.run(input_sender.clone(), output_receiver),
        main_task(&source, &sink, input_receiver, output_sender),
        server.run(input_sender.clone())
    )?;

    panic!("App exited unexpectedly")
}
