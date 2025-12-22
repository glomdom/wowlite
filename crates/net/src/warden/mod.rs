use tokio::io::{AsyncReadExt, BufReader};
use tracing::{debug, trace};

mod crypt;
mod error;
mod packet;
pub use crypt::WardenCrypt;
pub use error::WardenError;
pub use packet::WardenPacket;

use crate::Result;

pub struct Warden {
    crypt: WardenCrypt,
}

impl Warden {
    pub fn new(session_key: &[u8; 40]) -> Self {
        debug!("Initializing Warden emulation");

        Self {
            crypt: WardenCrypt::new(session_key),
        }
    }

    /// Handles an **encrypted** Warden packet.
    pub async fn decrypt_packet(&mut self, encrypted_data: Vec<u8>) -> Result<WardenPacket> {
        let decrypted = self.raw_decrypt_packet(encrypted_data);

        let mut r = BufReader::new(&*decrypted);
        let opcode = r.read_u8().await?;

        trace!("Handling Warden packet with opcode {}", opcode);

        match opcode {
            0x00 => {
                let mut module_id = [0u8; 16];
                r.read_exact(&mut module_id).await?;

                let mut module_key = [0u8; 16];
                r.read_exact(&mut module_key).await?;

                let size = r.read_u32_le().await?;

                Ok(WardenPacket::ModuleUse {
                    module_id,
                    module_key,
                    size,
                })
            }

            _ => Err(WardenError::InvalidWardenOpcode(opcode))?,
        }
    }

    pub fn encrypt_packet(&mut self, packet: WardenPacket) -> Vec<u8> {
        let mut data = packet.serialize();
        self.crypt.encrypt(&mut data);

        data
    }

    fn raw_decrypt_packet(&mut self, mut data: Vec<u8>) -> Vec<u8> {
        self.crypt.decrypt(&mut data);

        data
    }
}
