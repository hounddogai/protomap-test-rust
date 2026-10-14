use super::*;

use acme_kit::text;

/// Where a nonce variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl NonceVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for NonceVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a nonce variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceVariance {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub key_id: String,
    pub notes: String,
    pub quantity: i64,
    pub status: NonceVarianceStatus,
}

impl NonceVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceVarianceStatus) -> bool {
        match self.status {
            NonceVarianceStatus::Requested => {
                match next {
                    NonceVarianceStatus::Granted | NonceVarianceStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceVarianceStatus::Granted => {
                return next == NonceVarianceStatus::Revoked || next == NonceVarianceStatus::Expired;
            }
            NonceVarianceStatus::Revoked => {
                return next == NonceVarianceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce variance {} cannot move from {} to {}",
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
        self.quantity as f64
    }

    pub fn copy(&self) -> NonceVariance {
        NonceVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            key_id: self.key_id.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a nonce variance in status requested.
pub fn new_nonce_variance(code: &str, title: &str) -> NonceVariance {
    NonceVariance {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        key_id: "".to_string(),
        notes: "".to_string(),
        quantity: 0,
        status: NonceVarianceStatus::Requested,
    }
}
