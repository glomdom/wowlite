use thiserror::Error;
use wow_srp::error::NormalizedStringError;

#[derive(Error, Debug)]
pub enum NetError {
    #[error("Network IO error")]
    Io(#[from] std::io::Error),

    #[error("Normalized string error")]
    StringError(#[from] NormalizedStringError),

    #[error("Authentication failed")]
    Auth(#[from] crate::auth::AuthError),

    #[error("Warden error")]
    Warden(#[from] crate::warden::WardenError),

    #[error("Incomplete login packet")]
    IncompleteLoginPacket(#[from] wow_login_messages::errors::ExpectedOpcodeError),

    #[error("Incomplete world packet")]
    IncompleteWorldPacket(#[from] wow_world_messages::errors::ExpectedOpcodeError),
}

pub type Result<T> = std::result::Result<T, NetError>;
