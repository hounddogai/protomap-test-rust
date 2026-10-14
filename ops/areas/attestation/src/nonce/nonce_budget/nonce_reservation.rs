use super::*;

use acme_kit::text;

/// Where a nonce reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl NonceReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for NonceReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceReservation {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub name: String,
    pub priority: i64,
    pub key_id: String,
    pub visible: bool,
    pub status: NonceReservationStatus,
}

impl NonceReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceReservationStatus) -> bool {
        match self.status {
            NonceReservationStatus::Requested => {
                return next == NonceReservationStatus::Expired;
            }
            NonceReservationStatus::Granted => {
                match next {
                    NonceReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceReservationStatus::Revoked => {
                return next == NonceReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> NonceReservation {
        NonceReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            name: self.name.clone(),
            priority: self.priority,
            key_id: self.key_id.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a nonce reservation in status requested.
pub fn new_nonce_reservation(code: &str, title: &str) -> NonceReservation {
    NonceReservation {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        name: "".to_string(),
        priority: 0,
        key_id: "".to_string(),
        visible: false,
        status: NonceReservationStatus::Requested,
    }
}
