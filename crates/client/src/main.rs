use std::time::Duration;

use net::{
    event::{IncomingWorldNetworkEvent, OutgoingWorldNetworkEvent},
    world::{auth::authenticate_world, spawn_world_loop},
};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::mpsc};
use tracing::{debug, info, level_filters::LevelFilter, trace};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .from_env_lossy(),
        )
        .init();

    let (tx_world_in, _rx_world_in) = mpsc::channel::<IncomingWorldNetworkEvent>(4096);
    let (_tx_world_out, rx_world_out) = mpsc::channel::<OutgoingWorldNetworkEvent>(128);

    debug!("Initialized mpsc channels");
    info!("Connecting to auth server");

    let mut auth_stream = TcpStream::connect("127.0.0.1:3724").await?;
    // let mut auth_stream = TcpStream::connect("logon.turtle-server-eu.kz:3724").await?;
    let auth_session = net::auth::authenticate(&mut auth_stream, "test", "test123").await?;

    info!("Authenticated successfully, dropping connection to auth");

    auth_stream.shutdown().await?;

    debug!("Server has {} realms", auth_session.realms.len());

    let cur_realm = &auth_session.realms[0];
    trace!(
        "Attempting connection to {} ({})",
        cur_realm.name, cur_realm.address,
    );

    let mut world_stream = TcpStream::connect(&cur_realm.address).await?;
    info!("Connected to {}", cur_realm.name);

    let world_session = authenticate_world(&mut world_stream, &auth_session).await?;
    spawn_world_loop(world_stream, world_session, tx_world_in, rx_world_out);

    loop {
        std::thread::sleep(Duration::from_millis(100));
    }
}
