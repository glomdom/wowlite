#[derive(Debug)]
pub enum IncomingWorldNetworkEvent {
    Packet(Box<wow_world_messages::vanilla::opcodes::ServerOpcodeMessage>),
    Disconnected,
}

#[derive(Debug)]
pub enum OutgoingWorldNetworkEvent {
    Packet(Box<wow_world_messages::vanilla::opcodes::ClientOpcodeMessage>),
    Disconnected,
}
