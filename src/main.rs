use std::net::SocketAddr;

use proto::salesvc::sale_server::SaleServer;
use tokio::{
    net::TcpListener,
    signal::unix::{signal, SignalKind},
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tower_http::trace;
use tracing::Level;

use crate::{
    dependencies::Dependencies,
    sale::{tokens::TagManager, SaleApp},
};

mod config;
mod dependencies;
mod error;
pub mod proto;
mod sale;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let opt = envy::from_env::<config::Options>()?;
    let deps_config = envy::from_env::<config::DependencyConfig>()?;

    if deps_config.fake_price {
        tracing::warn!("fake price estimation enabled");
    }

    let deps = Dependencies::new(deps_config)?;
    let token_manager = TagManager::new(opt.token_secret.into_bytes());

    // build reflection service
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .with_service_name("salesvc.Sale")
        .build()?;

    // bind server socket
    let addr = SocketAddr::new(opt.ip, opt.port);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("starting server on {}", addr);

    Server::builder()
        // configure the server
        .layer(
            trace::TraceLayer::new_for_grpc()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        // enable grpc reflection
        .add_service(reflection)
        .add_service(SaleServer::new(SaleApp::new(deps, token_manager)))
        // serve
        .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
            let _ = signal(SignalKind::terminate()).unwrap().recv().await;
        })
        .await?;

    Ok(())
}
