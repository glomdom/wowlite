#[derive(Debug)]
pub enum WardenPacket {
    // S->C
    ModuleUse {
        module_id: [u8; 16],
        module_key: [u8; 16],
        size: u32,
    },

    ModuleOk {},
}

impl WardenPacket {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = vec![];

        match self {
            WardenPacket::ModuleUse {
                module_id,
                module_key,
                size,
            } => {
                buffer.push(1u8); // opcode
                buffer.extend_from_slice(module_id);
                buffer.extend_from_slice(module_key);
                buffer.extend_from_slice(&size.to_le_bytes());
            }

            WardenPacket::ModuleOk {} => {
                buffer.push(1u8);
            }
        }

        buffer
    }
}
