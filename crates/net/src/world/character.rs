use wow_world_messages::vanilla::{CMSG_CHAR_ENUM, opcodes::ClientOpcodeMessage};

pub fn get_characters_request() -> ClientOpcodeMessage {
    CMSG_CHAR_ENUM {}.into()
}
