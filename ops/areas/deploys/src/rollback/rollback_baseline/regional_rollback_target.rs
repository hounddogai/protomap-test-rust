use super::*;

use acme_kit::text;

/// Where a regional rollback target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRollbackTargetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalRollbackTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalRollbackTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional rollback target as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRollbackTarget {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub active: bool,
    pub scope: String,
    pub country: String,
    pub amount: f64,
    pub status: RegionalRollbackTargetStatus,
}

impl RegionalRollbackTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRollbackTargetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRollbackTargetStatus) -> bool {
        match self.status {
            RegionalRollbackTargetStatus::Proposed => {
                return next == RegionalRollbackTargetStatus::Rejected
                    || next == RegionalRollbackTargetStatus::Withdrawn;
            }
            RegionalRollbackTargetStatus::Approved => {
                return next == RegionalRollbackTargetStatus::Rejected
                    || next == RegionalRollbackTargetStatus::Withdrawn;
            }
            RegionalRollbackTargetStatus::Rejected => {
                match next {
                    RegionalRollbackTargetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalRollbackTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional rollback target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RegionalRollbackTarget {
        RegionalRollbackTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            active: self.active,
            scope: self.scope.clone(),
            country: self.country.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a regional rollback target in status proposed.
pub fn new_regional_rollback_target(code: &str, name: &str) -> RegionalRollbackTarget {
    RegionalRollbackTarget {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        active: false,
        scope: "".to_string(),
        country: "".to_string(),
        amount: 0.0,
        status: RegionalRollbackTargetStatus::Proposed,
    }
}
