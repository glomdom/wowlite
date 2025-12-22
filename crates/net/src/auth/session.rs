use wow_login_messages::version_3::Realm;
use wow_srp::{SESSION_KEY_LENGTH, normalized_string::NormalizedString};

pub struct AuthSession {
    pub session_key: [u8; SESSION_KEY_LENGTH as usize],
    pub realm_id: Option<u8>,
    pub realms: Vec<Realm>,
    pub username: NormalizedString,
}
