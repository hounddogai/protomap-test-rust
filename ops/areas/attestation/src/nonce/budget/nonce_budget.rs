use super::*;

use acme_kit::text;

/// Where a nonce budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl NonceBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for NonceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One nonce budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceBudget {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub description: String,
    pub segment: String,
    pub grace_days: i64,
    pub namespace: String,
    pub status: NonceBudgetStatus,
}

impl NonceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceBudgetStatus) -> bool {
        match self.status {
            NonceBudgetStatus::Proposed => {
                match next {
                    NonceBudgetStatus::Approved | NonceBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceBudgetStatus::Approved => {
                return next == NonceBudgetStatus::Rejected || next == NonceBudgetStatus::Withdrawn;
            }
            NonceBudgetStatus::Rejected => {
                return next == NonceBudgetStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> NonceBudget {
        NonceBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            description: self.description.clone(),
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a nonce budget in status proposed.
pub fn new_nonce_budget(code: &str, label: &str) -> NonceBudget {
    NonceBudget {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        description: "".to_string(),
        segment: "".to_string(),
        grace_days: 0,
        namespace: "".to_string(),
        status: NonceBudgetStatus::Proposed,
    }
}
