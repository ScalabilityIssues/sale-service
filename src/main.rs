use std::net::SocketAddr;

use proto::salesvc::sale_server::SaleServer;
use tokio::{
    net::TcpListener,
    signal::unix::{signal, SignalKind},
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Server};
use tower_http::trace;
use tracing::Level;

use crate::{dependencies::Dependencies, sale::SaleApp};

mod config;
mod dependencies;
pub mod proto;
mod sale;
mod error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let opt = envy::from_env::<config::Options>()?;

    tracing::info!("Loaded configuration: {:?}", opt);

    // define flightmngr grpc client
    let channel = Channel::builder(opt.flightmngr_url.try_into()?)
        .connect_lazy();

    // bind server socket
    let addr = SocketAddr::new(opt.ip, opt.port);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("starting server on {}", addr);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .with_service_name("salesvc.Sale")
        .build()?;

    Server::builder()
        // configure the server
        .timeout(std::time::Duration::from_secs(10))
        .layer(
            trace::TraceLayer::new_for_grpc()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        // enable grpc reflection
        .add_service(reflection)
        .add_service(SaleServer::new(SaleApp::new(Dependencies::new(channel))))
        // serve
        .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
            let _ = signal(SignalKind::terminate()).unwrap().recv().await;
        })
        .await?;

    Ok(())
}
