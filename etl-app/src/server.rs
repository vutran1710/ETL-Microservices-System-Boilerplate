use common::messages::Message;
use kanal::AsyncSender;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::{reply, Filter};

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    #[allow(dead_code)]
    fn with_sender<T>(
        query_sender: AsyncSender<T>,
    ) -> impl Filter<Extract = (AsyncSender<T>,), Error = Infallible> + Clone {
        warp::any().map(move || query_sender.clone())
    }

    async fn setup(&self) -> eyre::Result<()> {
        log::info!("Setting up server");
        Ok(())
    }

    async fn request_processing(
        request_message: Message,
        sender: AsyncSender<Message>,
    ) -> Result<impl warp::Reply, Infallible> {
        sender.send(request_message).await.expect("Failed to send");
        Ok(reply::with_status("OK", StatusCode::OK))
    }

    pub async fn run(&self, message_sender: AsyncSender<Message>) -> eyre::Result<()> {
        log::info!("Starting WebAPI server for application administrating");

        let health_check_route_root =
            warp::get().map(|| warp::reply::with_status("health check OK", StatusCode::OK));

        let request_processing_route = warp::post()
            .and(warp::path("process"))
            .and(warp::body::json())
            .and(Self::with_sender(message_sender.clone()))
            .and_then(Self::request_processing);

        log::info!("Starting HTTP server on port: {}", self.port);

        let routes = health_check_route_root.or(request_processing_route);

        tokio::try_join!(self.setup(), async {
            warp::serve(routes).run(([0, 0, 0, 0], self.port)).await;
            Ok(())
        })?;

        Ok(())
    }
}
