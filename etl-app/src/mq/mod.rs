use common::messages::Message;

#[cfg(feature = "google-cloud-pubsub")]
use google_cloud_pubsub::client::Client as PubSubClient;
#[cfg(feature = "google-cloud-pubsub")]
use google_cloud_pubsub::client::ClientConfig as PubSubClientConfig;

#[cfg(feature = "amqprs")]
mod rabbitmq;
#[cfg(feature = "amqprs")]
use rabbitmq::Args as RabbitMQArgs;
#[cfg(feature = "amqprs")]
use rabbitmq::RabbitMQ;

use async_trait::async_trait;
use clap::Parser;
use kanal::AsyncReceiver;
use kanal::AsyncSender;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct PubSubArgs {
    /// Google Cloud PubSub project ID
    #[arg(long)]
    pub project_id: String,

    /// Google Cloud PubSub client ID
    #[arg(long)]
    pub client_id: String,

    /// Google Cloud PubSub client secret
    #[arg(long)]
    pub client_secret: String,

    /// Google Cloud PubSub client refresh token
    #[arg(long)]
    pub refresh_token: String,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct QueueArgs {
    #[cfg(feature = "google-cloud-pubsub")]
    #[command(flatten, next_help_heading = "PubSub client")]
    pub pubsub: PubSubArgs,
    #[cfg(feature = "amqprs")]
    #[command(flatten, next_help_heading = "RabbitMQ client")]
    pub rabbitmq: RabbitMQArgs,
}

/*
MessageQueue should enable Auto-Ack
*/
pub enum MessageQueue {
    #[cfg(feature = "google-cloud-pubsub")]
    PubSub(PubSubClient),
    #[cfg(feature = "amqprs")]
    RabbitMQ(RabbitMQ),
}

#[async_trait]
pub trait MessageQueueTrait {
    async fn run(
        &self,
        source_sender: AsyncSender<Message>,
        sink_receiver: AsyncReceiver<Message>,
    ) -> eyre::Result<()>;
}

impl MessageQueue {
    pub async fn new(job_id: &str) -> eyre::Result<Self> {
        #[cfg(feature = "google-cloud-pubsub")]
        {
            let QueueArgs { pubsub } = QueueArgs::parse();
            let pubsub = pubsub.unwrap();
            let config = PubSubClientConfig::default().with_auth().await?;
            let client = PubSubClient::new(config).await?;
            return Ok(MessageQueue::PubSub(client));
        }

        #[cfg(feature = "amqprs")]
        {
            log::info!("Using RabbitMQ");
            let QueueArgs { rabbitmq: args } = QueueArgs::parse();
            let client = RabbitMQ::new(&args, job_id).await?;
            return Ok(MessageQueue::RabbitMQ(client));
        }
    }
}

#[async_trait]
impl MessageQueueTrait for MessageQueue {
    async fn run(
        &self,
        source_sender: AsyncSender<Message>,
        sink_receiver: AsyncReceiver<Message>,
    ) -> eyre::Result<()> {
        match self {
            #[cfg(feature = "google-cloud-pubsub")]
            MessageQueue::PubSub(client) => {
                log::info!("Running Google Cloud PubSub");
                unimplemented!()
            }
            #[cfg(feature = "amqprs")]
            MessageQueue::RabbitMQ(client) => client.run(source_sender, sink_receiver).await,
        }
    }
}
