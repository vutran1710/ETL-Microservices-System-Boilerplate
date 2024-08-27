mod mq;
mod server;

use clap::Parser;
use common::messages::Message;
use common::ETLTrait;

use kanal::AsyncReceiver;
use mq::MessageQueue;
use mq::MessageQueueTrait;
use server::Server;

#[cfg(feature = "action_job")]
use action_job::Etl;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        long,
        env = "ETL_SOURCE",
        default_value = "postgres://postgres:postgres@localhost:5432/mc2"
    )]
    source: String,

    #[arg(
        long,
        env = "ETL_SINK",
        default_value = "postgres://postgres:postgres@localhost:5432/mc2"
    )]
    sink: String,

    #[arg(
        long,
        env = "ETL_JOB_MANAGER",
        default_value = "postgres://postgres:postgres@localhost:5432/mc2"
    )]
    job_manager: String,

    #[arg(long, env = "ETL_SERVER_PORT", default_value = "8080")]
    port: u16,
}

async fn main_task(etl: Etl, receiver: AsyncReceiver<Message>) -> eyre::Result<()> {
    while let Ok(msg) = receiver.recv().await {
        etl.process_message_from_mq(msg).await?;
    }

    eyre::bail!("Message queue receiver exited unexpectedly")
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::try_init().ok();

    let Args {
        port,
        source,
        sink,
        job_manager,
    } = Args::parse();
    log::info!("Binding port: {}", port);

    let msg_queue = MessageQueue::new(&Etl::id()).await?;
    let server = Server::new(port);

    let (input_sender, input_receiver) = kanal::unbounded_async();
    let (output_sender, output_receiver) = kanal::unbounded_async();

    let etl = Etl::new(&source, &sink, &job_manager, output_sender)?;
    etl.resume().await?;

    tokio::try_join!(
        msg_queue.run(input_sender.clone(), output_receiver),
        main_task(etl, input_receiver),
        server.run(input_sender.clone())
    )?;

    panic!("App exited unexpectedly")
}
