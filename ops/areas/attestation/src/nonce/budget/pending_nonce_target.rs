use super::*;

use acme_kit::text;

/// Where a pending nonce target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingNonceTargetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingNonceTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingNonceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending nonce target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceTarget {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub pinned: bool,
    pub key_id: String,
    pub currency: String,
    pub tenant_id: String,
    pub priority: i64,
    pub status: PendingNonceTargetStatus,
}

impl PendingNonceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingNonceTargetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingNonceTargetStatus) -> bool {
        match self.status {
            PendingNonceTargetStatus::Proposed => {
                return next == PendingNonceTargetStatus::Rejected;
            }
            PendingNonceTargetStatus::Approved => {
                match next {
                    PendingNonceTargetStatus::Rejected | PendingNonceTargetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingNonceTargetStatus::Rejected => {
                match next {
                    PendingNonceTargetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingNonceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending nonce target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.floor_value + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingNonceTarget {
        PendingNonceTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            pinned: self.pinned,
            key_id: self.key_id.clone(),
            currency: self.currency.clone(),
            tenant_id: self.tenant_id.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending nonce target in status proposed.
pub fn new_pending_nonce_target(code: &str, name: &str) -> PendingNonceTarget {
    PendingNonceTarget {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        pinned: false,
        key_id: "".to_string(),
        currency: "".to_string(),
        tenant_id: "".to_string(),
        priority: 0,
        status: PendingNonceTargetStatus::Proposed,
    }
}
