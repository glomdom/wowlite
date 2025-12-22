use common::constants;
use std::net::Ipv4Addr;
use tokio::net::TcpStream;
use tracing::{info, trace, warn};

use wow_login_messages::Message;
use wow_login_messages::all::{
    CMD_AUTH_LOGON_CHALLENGE_Client, Locale, Os, Platform, ProtocolVersion, Version,
};
use wow_login_messages::helper::tokio_expect_server_message;
use wow_login_messages::version_2::{
    CMD_AUTH_LOGON_PROOF_Server, CMD_REALM_LIST_Client, CMD_REALM_LIST_Server,
};
use wow_login_messages::version_3::{
    CMD_AUTH_LOGON_CHALLENGE_Server, CMD_AUTH_LOGON_PROOF_Client,
    CMD_AUTH_LOGON_PROOF_Client_SecurityFlag,
};

use wow_srp::PublicKey;
use wow_srp::client::SrpClientChallenge;
use wow_srp::normalized_string::NormalizedString;

use crate::Result;
use crate::auth::{AuthError, AuthSession};

pub async fn authenticate(
    mut auth_server: &mut TcpStream,
    user: &str,
    pass: &str,
) -> Result<AuthSession> {
    info!("Attempting to authenticate with auth server");

    let username = NormalizedString::new(user)?;
    let password = NormalizedString::new(pass)?;

    CMD_AUTH_LOGON_CHALLENGE_Client {
        protocol_version: ProtocolVersion::Three,
        version: Version {
            major: 1,
            minor: 12,
            patch: 1,
            build: constants::WOW_REVISION,
        },
        platform: Platform::X86,
        os: Os::Windows,
        locale: Locale::EnGb,
        utc_timezone_offset: 120,
        client_ip_address: Ipv4Addr::LOCALHOST,
        account_name: username.to_string(),
    }
    .tokio_write(&mut auth_server)
    .await?;

    let s =
        tokio_expect_server_message::<CMD_AUTH_LOGON_CHALLENGE_Server, _>(&mut auth_server).await?;

    trace!("Received logon challenge");

    let c = if let CMD_AUTH_LOGON_CHALLENGE_Server::Success {
        generator,
        large_safe_prime,
        salt,
        server_public_key,
        ..
    } = s
    {
        let generator = generator[0];
        let large_safe_prime = large_safe_prime
            .try_into()
            .map_err(|_| AuthError::ServerCryptoInformation)?;

        let server_public_key = PublicKey::from_le_bytes(server_public_key)
            .map_err(|_| AuthError::ServerCryptoInformation)?;

        SrpClientChallenge::new(
            username.clone(),
            password,
            generator,
            large_safe_prime,
            server_public_key,
            salt,
        )
    } else {
        warn!(
            "TODO: Proper error handling for logon challenge response. Got {:?}",
            s
        );

        return Err(AuthError::ServerCryptoInformation)?;
    };

    trace!("Created client SRP");

    CMD_AUTH_LOGON_PROOF_Client {
        client_public_key: *c.client_public_key(),
        client_proof: *c.client_proof(),
        crc_hash: [0u8; 20],
        telemetry_keys: vec![],
        security_flag: CMD_AUTH_LOGON_PROOF_Client_SecurityFlag::None,
    }
    .tokio_write(&mut auth_server)
    .await?;

    let s = tokio_expect_server_message::<CMD_AUTH_LOGON_PROOF_Server, _>(&mut auth_server).await?;
    let c = if let CMD_AUTH_LOGON_PROOF_Server::Success { server_proof, .. } = s {
        c.verify_server_proof(server_proof)
            .map_err(|_| AuthError::ServerVerificationFailed)?
    } else {
        return Err(AuthError::ServerVerificationFailed)?;
    };

    CMD_REALM_LIST_Client {}
        .tokio_write(&mut auth_server)
        .await?;

    let realms = tokio_expect_server_message::<CMD_REALM_LIST_Server, _>(&mut auth_server).await?;
    let session = AuthSession {
        session_key: *c.session_key(),
        realm_id: None,
        realms: realms.realms,
        username,
    };

    trace!("Received realm list");

    Ok(session)
}
