use common::messages::Message;

#[cfg(feature = "pubsub")]
use google_cloud_pubsub::client::Client as PubSubClient;
#[cfg(feature = "pubsub")]
use google_cloud_pubsub::client::ClientConfig as PubSubClientConfig;

#[cfg(feature = "rabbitmq")]
mod rabbitmq;
#[cfg(feature = "rabbitmq")]
use rabbitmq::Args as RabbitMQArgs;
#[cfg(feature = "rabbitmq")]
use rabbitmq::RabbitMQ;

use async_trait::async_trait;
use clap::Parser;
use kanal::AsyncReceiver;
use kanal::AsyncSender;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct PubSubArgs {
    /// Google Cloud PubSub project ID
    #[arg(short, long)]
    pub project_id: String,

    /// Google Cloud PubSub client ID
    #[arg(short, long)]
    pub client_id: String,

    /// Google Cloud PubSub client secret
    #[arg(short, long)]
    pub client_secret: String,

    /// Google Cloud PubSub client refresh token
    #[arg(short, long)]
    pub refresh_token: String,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct QueueArgs {
    /// Database connect options
    #[command(flatten, next_help_heading = "PubSub client")]
    pub pubsub: Option<PubSubArgs>,
    #[command(flatten, next_help_heading = "RabbitMQ client")]
    pub rabbitmq: Option<RabbitMQArgs>,
}

pub enum MessageQueue {
    #[cfg(feature = "pubsub")]
    PubSub(PubSubClient),
    #[cfg(feature = "rabbitmq")]
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
    pub async fn new() -> eyre::Result<Self> {
        let QueueArgs { pubsub, rabbitmq } = QueueArgs::parse();

        #[allow(dead_code)]
        if let Some(_pubsub_args) = pubsub {
            #[cfg(feature = "pubsub")]
            {
                log::info!("Using Google Cloud PubSub");
                let config = PubSubClientConfig::default().with_auth().await?;
                let client = PubSubClient::new(config).await?;
                return Ok(MessageQueue::PubSub(client));
            }
        }

        #[allow(dead_code)]
        if let Some(_rabbitmq_args) = rabbitmq {
            #[cfg(feature = "rabbitmq")]
            {
                log::info!("Using RabbitMQ");
                let client = RabbitMQ::new(&_rabbitmq_args).await?;
                return Ok(MessageQueue::RabbitMQ(client));
            }
        }

        unimplemented!("No message queue client specified")
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
            #[cfg(feature = "pubsub")]
            MessageQueue::PubSub(client) => {
                log::info!("Running Google Cloud PubSub");
                unimplemented!()
            }
            #[cfg(feature = "rabbitmq")]
            MessageQueue::RabbitMQ(client) => client.run(source_sender, sink_receiver).await,
        }
    }
}
