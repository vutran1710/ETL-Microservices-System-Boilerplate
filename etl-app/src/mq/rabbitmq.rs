use super::MessageQueueTrait;

use amqprs::callbacks::DefaultConnectionCallback;
use amqprs::channel::BasicConsumeArguments;
use amqprs::channel::BasicPublishArguments;
use amqprs::channel::ExchangeDeclareArguments;
use amqprs::channel::QueueBindArguments;
use amqprs::channel::QueueDeclareArguments;
use amqprs::channel::{BasicAckArguments, Channel};
use amqprs::connection::Connection;
use amqprs::connection::OpenConnectionArguments;
use amqprs::consumer::AsyncConsumer;
use amqprs::BasicProperties;
use amqprs::Deliver;
use async_trait::async_trait;
use clap::Parser;
use common::messages::Message;
use eyre::Result;
use kanal::AsyncReceiver;
use kanal::AsyncSender;
use tokio::select;

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, env = "RABBITMQ_EXCHANGE", default_value = "etl_exchange")]
    pub exchange: String,
    #[arg(long, env = "RABBITMQ_SOURCE_QUEUE", default_value = "etl_tier_1")]
    pub source_queue: String,
    #[arg(long, env = "RABBITMQ_SINK_QUEUE", default_value = "etl_tier_2")]
    pub sink_queue: String,
    #[arg(long, env = "RABBITMQ_HOST", default_value = "localhost")]
    pub host: String,
    #[arg(long, env = "RABBITMQ_USERNAME", default_value = "guest")]
    pub username: String,
    #[arg(long, env = "RABBITMQ_PASSWORD", default_value = "guest")]
    pub password: String,
}

pub struct RabbitMQ {
    connection: Connection,
    args: Args,
}

struct RabbitMqConsumer {
    sender: AsyncSender<Message>,
}

#[async_trait]
impl AsyncConsumer for RabbitMqConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        delivery: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let message = String::from_utf8(content).unwrap();
        log::info!("Received message: {}", message);
        if let Ok(msg) = serde_json::from_str::<Message>(&message) {
            log::info!("Valid message found: {}", msg);
            self.sender.send(msg).await.unwrap();
        }

        // FIXME: Acknowledge the message, only after processing it
        channel
            .basic_ack(BasicAckArguments::new(delivery.delivery_tag(), false))
            .await
            .expect("Failed to acknowledge message");
    }
}

impl RabbitMQ {
    pub async fn new(args: &Args) -> Result<Self> {
        let conn_args =
            OpenConnectionArguments::new(&args.host, 5672, &args.username, &args.password);
        let connection = Connection::open(&conn_args).await?;
        connection
            .register_callback(DefaultConnectionCallback)
            .await?;

        Ok(Self {
            connection,
            args: args.to_owned(),
        })
    }
}

impl RabbitMQ {
    async fn create_channel(&self, queue: &str, routing_key: &str) -> eyre::Result<Channel> {
        let channel = self.connection.open_channel(None).await?;
        channel
            .exchange_declare(ExchangeDeclareArguments::new(&self.args.exchange, "direct"))
            .await?;
        channel
            .queue_declare(QueueDeclareArguments::new(queue))
            .await?;
        channel
            .queue_bind(QueueBindArguments::new(
                queue,
                &self.args.exchange,
                routing_key,
            ))
            .await?;
        Ok(channel)
    }
}

#[async_trait]
impl MessageQueueTrait for RabbitMQ {
    async fn run(
        &self,
        source_sender: AsyncSender<Message>,
        sink_receiver: AsyncReceiver<Message>,
    ) -> Result<()> {
        // Consuming message

        let task_consume = || async move {
            let channel = self
                .create_channel(&self.args.source_queue, "tier_1")
                .await
                .unwrap();

            // FIXME: dynamic consumer name
            let consumer_tag = "elt-source-consumer";
            let consume_args = BasicConsumeArguments::new(&self.args.source_queue, consumer_tag);
            channel
                .basic_consume(
                    RabbitMqConsumer {
                        sender: source_sender.clone(),
                    },
                    consume_args,
                )
                .await
                .expect("Failed to start consuming messages");

            tokio::task::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                }
            })
            .await
            .expect("Task failed");
        };

        // Publishing message
        let task_publish = || async move {
            let channel = self
                .create_channel(&self.args.sink_queue, "tier_2")
                .await
                .unwrap();
            let publish_args = BasicPublishArguments::new(&self.args.exchange, "");

            while let Ok(msg) = sink_receiver.recv().await {
                let message = serde_json::to_string(&msg).unwrap();
                channel
                    .basic_publish(
                        BasicProperties::default(),
                        message.as_bytes().to_vec(),
                        publish_args.clone(),
                    )
                    .await
                    .expect("Failed to publish message");
            }
        };

        select! {
            _ = task_consume() => {},
            _ = task_publish() => {},
        }

        eyre::bail!("RabbitMQ exited unexpectedly")
    }
}
