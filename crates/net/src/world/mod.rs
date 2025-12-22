use tokio::{net::TcpStream, sync::mpsc};
use tracing::{debug, error, trace};
use wow_world_messages::vanilla::opcodes::ServerOpcodeMessage;

use crate::{
    event::{IncomingWorldNetworkEvent, OutgoingWorldNetworkEvent},
    world::session::WorldSession,
};

pub mod auth;
pub mod session;

pub fn spawn_world_loop(
    mut stream: TcpStream,
    mut world_session: WorldSession,
    tx_world: mpsc::Sender<IncomingWorldNetworkEvent>,
    mut rx_world: mpsc::Receiver<OutgoingWorldNetworkEvent>,
) {
    tokio::spawn(async move {
        trace!("Started world network loop");

        loop {
            tokio::select! {
                msg = ServerOpcodeMessage::tokio_read_encrypted(&mut stream, world_session.crypto.decrypter()) => {
                    match msg {
                        Ok(packet) => {
                            if tx_world.send(IncomingWorldNetworkEvent::Packet(Box::new(packet))).await.is_err() {
                                error!("Game thread is dead, cannot send incoming world packet");

                                break;
                            }
                        }

                        Err(e) => {
                            error!("Read error in network loop: {}", e);

                            std::mem::drop(tx_world.send(IncomingWorldNetworkEvent::Disconnected));

                            break;
                        }
                    }
                }

                Some(event) = rx_world.recv() => {
                    match event {
                        OutgoingWorldNetworkEvent::Packet(packet) => {
                            if let Err(e) = packet.tokio_write_encrypted_server(&mut stream, world_session.crypto.encrypter()).await {
                                error!("Write error in network loop: {}", e);

                                break;
                            }
                        }

                        OutgoingWorldNetworkEvent::Disconnected => {
                            debug!("Game requested disconnect");

                            break;
                        }
                    }
                }
            }
        }
    });
}
