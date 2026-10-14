use super::*;

use acme_kit::text;

/// Where a approval budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ApprovalBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ApprovalBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBudget {
    pub code: String,
    pub label: String,
    pub region: String,
    pub capacity: i64,
    pub segment: String,
    pub sequence: i64,
    pub status: ApprovalBudgetStatus,
}

impl ApprovalBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalBudgetStatus) -> bool {
        match self.status {
            ApprovalBudgetStatus::Requested => {
                match next {
                    ApprovalBudgetStatus::Granted | ApprovalBudgetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalBudgetStatus::Granted => {
                match next {
                    ApprovalBudgetStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalBudgetStatus::Revoked => {
                return next == ApprovalBudgetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.capacity as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalBudget {
        ApprovalBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            capacity: self.capacity,
            segment: self.segment.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a approval budget in status requested.
pub fn new_approval_budget(code: &str, label: &str) -> ApprovalBudget {
    ApprovalBudget {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        capacity: 0,
        segment: "".to_string(),
        sequence: 0,
        status: ApprovalBudgetStatus::Requested,
    }
}
