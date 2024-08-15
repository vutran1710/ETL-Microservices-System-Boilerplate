use common::messages::Message;
use kanal::AsyncSender;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Filter;

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

    pub async fn run(&self, _message_sender: AsyncSender<Message>) -> eyre::Result<()> {
        log::info!("Starting WebAPI server for application administrating");

        let health_check_route_root =
            warp::get().map(|| warp::reply::with_status("health check OK", StatusCode::OK));

        log::info!("Starting HTTP server on port: {}", self.port);

        let routes = health_check_route_root;

        tokio::try_join!(self.setup(), async {
            warp::serve(routes).run(([0, 0, 0, 0], self.port)).await;
            Ok(())
        })?;

        Ok(())
    }
}
