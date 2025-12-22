use common::constants;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tracing::{info, trace, warn};

use wow_srp::vanilla_header::ProofSeed;
use wow_world_messages::vanilla::{
    AddonInfo, CMSG_AUTH_SESSION, CMSG_WARDEN_DATA, ClientMessage, SMSG_AUTH_CHALLENGE,
    tokio_expect_server_message,
};

use crate::{
    Result,
    auth::AuthSession,
    warden::{Warden, WardenPacket},
    world::session::WorldSession,
};

pub async fn authenticate_world(
    mut world_stream: &mut TcpStream,
    auth_session: &AuthSession,
) -> Result<WorldSession> {
    let s = tokio_expect_server_message::<SMSG_AUTH_CHALLENGE, _>(&mut world_stream).await?;

    let seed = ProofSeed::new();
    let seed_value = seed.seed();
    let (client_proof, mut crypto) = seed.into_client_header_crypto(
        &auth_session.username,
        auth_session.session_key,
        s.server_seed,
    );

    let mut warden = Warden::new(&auth_session.session_key);

    let addons = vec![
        create_blizzard_addon_info("Blizzard_AuctionUI"),
        create_blizzard_addon_info("Blizzard_BattlefieldMinimap"),
        create_blizzard_addon_info("Blizzard_CraftUI"),
        create_blizzard_addon_info("Blizzard_InspectUI"),
        create_blizzard_addon_info("Blizzard_MacroUI"),
        create_blizzard_addon_info("Blizzard_RaidUI"),
        create_blizzard_addon_info("Blizzard_TalentUI"),
        create_blizzard_addon_info("Blizzard_TradeSkillUI"),
        create_blizzard_addon_info("Blizzard_TrainerUI"),
        create_addon_info("Turtle_General", 1101498516),
    ];

    trace!("Initialized fake addons");

    CMSG_AUTH_SESSION {
        build: constants::REAL_WOW_REVISION as u32,
        server_id: auth_session.realm_id.unwrap_or(0) as u32,
        username: auth_session.username.to_string(),
        client_seed: seed_value,
        client_proof,
        addon_info: addons,
    }
    .tokio_write_unencrypted_client(&mut world_stream)
    .await?;

    loop {
        let mut header = [0_u8; 4];
        world_stream.read_exact(&mut header).await?;
        let header = crypto.decrypt_server_header(header);
        let opcode = header.opcode;
        let body_size = (header.size.saturating_sub(2)) as u32;

        let mut buf = vec![0; body_size as usize];
        world_stream.read_exact(&mut buf).await?;

        match opcode {
            0x02E6 => {
                let data = warden.decrypt_packet(buf).await?;

                let output = match data {
                    WardenPacket::ModuleUse { .. } => {
                        warn!("TODO: Properly download module and inspect how it works");
                        let mod_ok = WardenPacket::ModuleOk {};

                        warden.encrypt_packet(mod_ok)
                    }

                    _ => panic!("Unhandled warden packet"),
                };

                CMSG_WARDEN_DATA {
                    encrypted_data: output,
                }
                .tokio_write_encrypted_client(&mut world_stream, crypto.encrypter())
                .await?;

                trace!("Sent module OK to server");
            }

            0x02EF => {
                warn!("Skipping addon packet of size {}", buf.len());
            }

            0x01EE => {
                let r = buf[0];

                match r {
                    12 => break,
                    _ => panic!("Nope"),
                }
            }

            other => {
                warn!("Unexpected packet during handshake: {:?}", other);
            }
        }
    }

    info!("Authenticated to world server successfully");

    let session = WorldSession {
        warden,
        crypto,
        seed: seed_value,
        proof: client_proof,
    };

    Ok(session)
}

fn create_addon_info(name: &str, crc: u32) -> AddonInfo {
    AddonInfo {
        addon_name: name.to_owned(),
        addon_has_signature: 0,
        addon_crc: crc,
        addon_extra_crc: 0,
    }
}

fn create_blizzard_addon_info(name: &str) -> AddonInfo {
    AddonInfo {
        addon_name: name.to_owned(),
        addon_has_signature: 0,
        addon_crc: 1276933997,
        addon_extra_crc: 0,
    }
}
