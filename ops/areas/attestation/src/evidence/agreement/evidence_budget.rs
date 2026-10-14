use super::*;

use acme_kit::text;

/// Where a evidence budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EvidenceBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for EvidenceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBudget {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub owner_id: String,
    pub rotation_days: i64,
    pub phone: String,
    pub status: EvidenceBudgetStatus,
}

impl EvidenceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceBudgetStatus) -> bool {
        match self.status {
            EvidenceBudgetStatus::Draft => {
                return next == EvidenceBudgetStatus::Suspended;
            }
            EvidenceBudgetStatus::Pending => {
                return next == EvidenceBudgetStatus::Suspended || next == EvidenceBudgetStatus::Closed;
            }
            EvidenceBudgetStatus::Active => {
                return next == EvidenceBudgetStatus::Closed;
            }
            EvidenceBudgetStatus::Suspended => {
                return next == EvidenceBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "evidence budget", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> EvidenceBudget {
        EvidenceBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            owner_id: self.owner_id.clone(),
            rotation_days: self.rotation_days,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a evidence budget in status draft.
pub fn new_evidence_budget(code: &str, label: &str) -> EvidenceBudget {
    EvidenceBudget {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        owner_id: "".to_string(),
        rotation_days: 0,
        phone: "".to_string(),
        status: EvidenceBudgetStatus::Draft,
    }
}
