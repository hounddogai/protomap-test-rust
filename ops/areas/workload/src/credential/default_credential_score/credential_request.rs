use super::*;

use acme_kit::text;

/// Where a credential request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CredentialRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CredentialRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential request as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRequest {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub phone: String,
    pub batch_size: i64,
    pub floor_value: f64,
    pub sequence: i64,
    pub cipher: String,
    pub attempts: i64,
    pub port: i64,
    pub rotation_days: i64,
    pub status: CredentialRequestStatus,
}

impl CredentialRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialRequestStatus) -> bool {
        match self.status {
            CredentialRequestStatus::Requested => {
                return next == CredentialRequestStatus::Granted || next == CredentialRequestStatus::Expired;
            }
            CredentialRequestStatus::Granted => {
                match next {
                    CredentialRequestStatus::Revoked | CredentialRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialRequestStatus::Revoked => {
                return next == CredentialRequestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialRequest {
        CredentialRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            phone: self.phone.clone(),
            batch_size: self.batch_size,
            floor_value: self.floor_value,
            sequence: self.sequence,
            cipher: self.cipher.clone(),
            attempts: self.attempts,
            port: self.port,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a credential request in status requested.
pub fn new_credential_request(code: &str, label: &str) -> CredentialRequest {
    CredentialRequest {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        phone: "".to_string(),
        batch_size: 0,
        floor_value: 0.0,
        sequence: 0,
        cipher: "".to_string(),
        attempts: 0,
        port: 0,
        rotation_days: 0,
        status: CredentialRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_credential_request("code-156", "label-900");
        assert_eq!(item.status, CredentialRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_credential_request("code-156", "label-900");
        if item.move_to(CredentialRequestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialRequestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_credential_request("code-156", "label-900");
        item.status = CredentialRequestStatus::Expired;
        assert!(
            item.move_to(CredentialRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
