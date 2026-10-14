use super::*;

use acme_kit::text;

/// Where a credential reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CredentialReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CredentialReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReservation {
    pub code: String,
    pub name: String,
    pub region: String,
    pub minimum: f64,
    pub quantity: i64,
    pub grace_days: i64,
    pub host: String,
    pub notes: String,
    pub reviewer: String,
    pub status: CredentialReservationStatus,
}

impl CredentialReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialReservationStatus) -> bool {
        match self.status {
            CredentialReservationStatus::Requested => {
                match next {
                    CredentialReservationStatus::Revoked | CredentialReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialReservationStatus::Granted => {
                return next == CredentialReservationStatus::Expired;
            }
            CredentialReservationStatus::Revoked => {
                return next == CredentialReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialReservation {
        CredentialReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            minimum: self.minimum,
            quantity: self.quantity,
            grace_days: self.grace_days,
            host: self.host.clone(),
            notes: self.notes.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a credential reservation in status requested.
pub fn new_credential_reservation(code: &str, name: &str) -> CredentialReservation {
    CredentialReservation {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        minimum: 0.0,
        quantity: 0,
        grace_days: 0,
        host: "".to_string(),
        notes: "".to_string(),
        reviewer: "".to_string(),
        status: CredentialReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_credential_reservation("code-67", "name-253");
        assert_eq!(item.status, CredentialReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_credential_reservation("code-67", "name-253");
        if item.move_to(CredentialReservationStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialReservationStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_credential_reservation("code-67", "name-253");
        item.status = CredentialReservationStatus::Expired;
        assert!(
            item.move_to(CredentialReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
