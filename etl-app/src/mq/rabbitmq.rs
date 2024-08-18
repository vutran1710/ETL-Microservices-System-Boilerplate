use super::MessageQueueTrait;
use amqprs::callbacks::DefaultChannelCallback;
use amqprs::callbacks::DefaultConnectionCallback;
use amqprs::channel::BasicConsumeArguments;
use amqprs::channel::BasicPublishArguments;
use amqprs::channel::QueueDeclareArguments;
use amqprs::connection::Connection;
use amqprs::connection::OpenConnectionArguments;
use amqprs::consumer::DefaultConsumer;
use amqprs::BasicProperties;
use async_trait::async_trait;
use clap::Parser;
use common::messages::Message;
use eyre::Result;
use kanal::AsyncReceiver;
use kanal::AsyncSender;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, env = "RABBITMQ_SOURCE_QUEUE")]
    pub source_queue: String,
    #[arg(long, env = "RABBITMQ_SINK_QUEUE")]
    pub sink_queue: String,
    #[arg(long, env = "RABBITMQ_HOST", default_value = "localhost")]
    pub host: String,
    #[arg(long, env = "RABBITMQ_USERNAME", default_value = "guest")]
    pub username: String,
    #[arg(long, env = "RABBITMQ_PASSWORD", default_value = "guest")]
    pub password: String,
}

pub struct RabbitMQ {
    source_channel: amqprs::channel::Channel,
    sink_channel: amqprs::channel::Channel,
    source_queue: String,
    sink_queue: String,
}

impl RabbitMQ {
    pub async fn new(args: &Args) -> Result<Self> {
        let conn_args =
            OpenConnectionArguments::new(&args.host, 5672, &args.username, &args.password);
        let connection = Connection::open(&conn_args).await?;
        connection
            .register_callback(DefaultConnectionCallback)
            .await?;

        let source_channel = connection.open_channel(None).await?;
        source_channel
            .register_callback(DefaultChannelCallback)
            .await?;

        let queue_args = QueueDeclareArguments::default()
            .queue(args.source_queue.clone())
            .finish();
        source_channel.queue_declare(queue_args).await?;

        let sink_channel = connection.open_channel(None).await?;
        sink_channel
            .register_callback(DefaultChannelCallback)
            .await?;

        let queue_args = QueueDeclareArguments::default()
            .queue(args.sink_queue.clone())
            .finish();
        sink_channel.queue_declare(queue_args).await?;

        Ok(Self {
            source_channel,
            sink_channel,
            source_queue: args.source_queue.clone(),
            sink_queue: args.sink_queue.clone(),
        })
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
        let args = BasicConsumeArguments::new(&self.source_queue, "etl-mq-consumer");
        let no_ack = args.no_ack;

        let task_consume = || async move {
            while let Ok(message_as_str) = self
                .source_channel
                .basic_consume(DefaultConsumer::new(no_ack), args.clone())
                .await
            {
                let msg: Message =
                    serde_json::from_str(&message_as_str).expect("Failed to deserialize message");
                source_sender.send(msg).await.unwrap()
            }
            eyre::bail!("Failed received msg");
            #[allow(unreachable_code)]
            Ok::<(), eyre::Error>(()) // NOTE: just for the compiler to know the return type
        };

        // Publishing message
        let task_publish = || async move {
            let args = BasicPublishArguments::new(&self.sink_queue, "");
            let properties = BasicProperties::default();

            while let Ok(message) = sink_receiver.recv().await {
                let message = serde_json::to_string(&message).expect("Failed to serialize message");

                self.sink_channel
                    .basic_publish(properties.clone(), message.into_bytes(), args.clone())
                    .await
                    .expect("Failed to publish message to MQ")
            }
            eyre::bail!("Sink receiver closed");
            #[allow(unreachable_code)]
            Ok::<(), eyre::Error>(()) // NOTE: just for the compiler to know the return type
        };

        tokio::try_join!(task_consume(), task_publish())?;

        eyre::bail!("RabbitMQ exited unexpectedly")
    }
}
