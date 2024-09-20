use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailAccount {
    pub email: String,
    pub password: String,
    pub imap: String,
}

pub trait LoadAccount {
    fn load_account(email: &str, password: &str, imap: &str) -> EmailAccount;
}
