mod mq;
mod server;

use clap::Parser;
use common::messages::Message;
use common::ETLTrait;

use kanal::AsyncReceiver;
use kanal::AsyncSender;
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
        default_value = "postgres://postgres:postgres@localhost:5432/postgres"
    )]
    source: String,

    #[arg(
        long,
        env = "ETL_SINK",
        default_value = "postgres://postgres:postgres@localhost:5432/postgres"
    )]
    sink: String,

    #[arg(
        long,
        env = "ETL_JOB_MANAGER",
        default_value = "postgres://postgres:postgres@localhost:5432/postgres"
    )]
    job_manager: String,

    #[arg(long, env = "ETL_SERVER_PORT", default_value = "8080")]
    port: u16,
}

async fn main_task(
    source: &str,
    sink: &str,
    job_manager: &str,
    receiver: AsyncReceiver<Message>,
    emitter: AsyncSender<Message>,
) -> eyre::Result<()> {
    let etl = Etl::new(source, sink, job_manager).await?;

    while let Ok(msg) = receiver.recv().await {
        etl.process_message(msg, emitter.clone()).await?;
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
    let msg_queue = MessageQueue::new().await?;
    let server = Server::new(port);

    let (input_sender, input_receiver) = kanal::unbounded_async();
    let (output_sender, output_receiver) = kanal::unbounded_async();

    tokio::try_join!(
        msg_queue.run(input_sender.clone(), output_receiver),
        main_task(&source, &sink, &job_manager, input_receiver, output_sender),
        server.run(input_sender.clone())
    )?;

    panic!("App exited unexpectedly")
}
