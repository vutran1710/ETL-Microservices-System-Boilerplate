use common::messages::Message;
#[cfg(feature = "pubsub")]
use google_cloud_pubsub::client::Client as PubSubClient;
#[cfg(feature = "pubsub")]
use google_cloud_pubsub::client::ClientConfig as PubSubClientConfig;

use clap::Parser;
use database::interfaces::OrderingID;
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
}

pub enum MessageQueue {
    #[cfg(feature = "pubsub")]
    PubSub(PubSubClient),
}

impl MessageQueue {
    pub async fn new() -> eyre::Result<Self> {
        let QueueArgs { pubsub } = QueueArgs::parse();

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

        unimplemented!("No message queue client specified")
    }

    pub async fn run<T: OrderingID, P: OrderingID>(
        &self,
        _source_sender: AsyncSender<Message<T>>,
        _sink_receiver: AsyncReceiver<Message<P>>,
    ) -> eyre::Result<()> {
        match self {
            #[cfg(feature = "pubsub")]
            MessageQueue::PubSub(_client) => {
                todo!("Implement the run function for Google Cloud PubSub")
            }
        }
    }
}
