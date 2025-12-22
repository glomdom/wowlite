use thiserror::Error;
use wow_srp::error::MatchProofsError;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Proofs did not match")]
    ProofsMismatch(#[from] MatchProofsError),

    #[error("Server verification failed")]
    ServerVerificationFailed,

    #[error("Server sent invalid crypto information")]
    ServerCryptoInformation,
}
