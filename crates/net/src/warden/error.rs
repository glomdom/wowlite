use thiserror::Error;

#[derive(Error, Debug)]
pub enum WardenError {
    #[error("Invalid Warden opcode {0}")]
    InvalidWardenOpcode(u8),
}
