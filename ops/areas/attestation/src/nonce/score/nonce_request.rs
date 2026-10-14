use super::*;

use acme_kit::text;

/// Where a nonce request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl NonceRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for NonceRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One nonce request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceRequest {
    pub code: String,
    pub label: String,
    pub title: String,
    pub minimum: f64,
    pub quantity: i64,
    pub budget: f64,
    pub status: NonceRequestStatus,
}

impl NonceRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceRequestStatus) -> bool {
        match self.status {
            NonceRequestStatus::Proposed => {
                match next {
                    NonceRequestStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceRequestStatus::Approved => {
                match next {
                    NonceRequestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceRequestStatus::Rejected => {
                return next == NonceRequestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "nonce request", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.budget + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceRequest {
        NonceRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            quantity: self.quantity,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a nonce request in status proposed.
pub fn new_nonce_request(code: &str, label: &str) -> NonceRequest {
    NonceRequest {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        minimum: 0.0,
        quantity: 0,
        budget: 0.0,
        status: NonceRequestStatus::Proposed,
    }
}
