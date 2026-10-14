use super::*;

use acme_kit::text;

/// Where a grant allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GrantAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GrantAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantAllocation {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub owner_id: String,
    pub description: String,
    pub amount: f64,
    pub status: GrantAllocationStatus,
}

impl GrantAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantAllocationStatus) -> bool {
        match self.status {
            GrantAllocationStatus::Proposed => {
                return next == GrantAllocationStatus::Withdrawn;
            }
            GrantAllocationStatus::Approved => {
                match next {
                    GrantAllocationStatus::Rejected | GrantAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantAllocationStatus::Rejected => {
                match next {
                    GrantAllocationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: GrantAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "grant allocation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> GrantAllocation {
        GrantAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            owner_id: self.owner_id.clone(),
            description: self.description.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a grant allocation in status proposed.
pub fn new_grant_allocation(code: &str, label: &str) -> GrantAllocation {
    GrantAllocation {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        owner_id: "".to_string(),
        description: "".to_string(),
        amount: 0.0,
        status: GrantAllocationStatus::Proposed,
    }
}
