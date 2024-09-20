use super::account::*;

impl LoadAccount for EmailAccount {
    fn load_account(email: &str, password: &str, imap: &str) -> EmailAccount {
        EmailAccount {
            email: email.to_string(),
            password: password.to_string(),
            imap: imap.to_string(),
        }
    }
}
