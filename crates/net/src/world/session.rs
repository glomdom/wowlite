use tracing::debug;
use wow_srp::{
    PROOF_LENGTH,
    vanilla_header::{HeaderCrypto, ProofSeed},
};

use crate::{auth::AuthSession, warden::Warden};

/// A session which contains a client session's Warden, Header (de|en)cryptor **(which is invalid until authenticated with the world server)**, seed and proof.
pub struct WorldSession {
    pub warden: Warden,
    pub crypto: HeaderCrypto,
    pub seed: u32,
    pub proof: [u8; PROOF_LENGTH as usize],
}

impl WorldSession {
    pub fn new(auth_session: &AuthSession, server_seed: u32) -> Self {
        let seed = ProofSeed::new();
        let seed_value = seed.seed();
        let (client_proof, crypto) = seed.into_client_header_crypto(
            &auth_session.username,
            auth_session.session_key,
            server_seed,
        );

        debug!("Initialized world session with client seed {}", seed_value);

        let warden = Warden::new(&auth_session.session_key);

        Self {
            warden,
            crypto,
            seed: seed_value,
            proof: client_proof,
        }
    }
}
