use super::*;

use acme_kit::text;

/// Where a primary migration budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryMigrationBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryMigrationBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryMigrationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary migration budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMigrationBudget {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub threshold: f64,
    pub margin: f64,
    pub segment: String,
    pub principal: String,
    pub attempts: i64,
    pub rate: f64,
    pub status: PrimaryMigrationBudgetStatus,
}

impl PrimaryMigrationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryMigrationBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryMigrationBudgetStatus) -> bool {
        match self.status {
            PrimaryMigrationBudgetStatus::Requested => {
                return next == PrimaryMigrationBudgetStatus::Revoked || next == PrimaryMigrationBudgetStatus::Expired;
            }
            PrimaryMigrationBudgetStatus::Granted => {
                match next {
                    PrimaryMigrationBudgetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryMigrationBudgetStatus::Revoked => {
                match next {
                    PrimaryMigrationBudgetStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryMigrationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary migration budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.threshold - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryMigrationBudget {
        PrimaryMigrationBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            threshold: self.threshold,
            margin: self.margin,
            segment: self.segment.clone(),
            principal: self.principal.clone(),
            attempts: self.attempts,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a primary migration budget in status requested.
pub fn new_primary_migration_budget(code: &str, name: &str) -> PrimaryMigrationBudget {
    PrimaryMigrationBudget {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        threshold: 0.0,
        margin: 0.0,
        segment: "".to_string(),
        principal: "".to_string(),
        attempts: 0,
        rate: 0.0,
        status: PrimaryMigrationBudgetStatus::Requested,
    }
}
