use super::*;

use acme_kit::text;

/// Where a tiered budget burn score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredBudgetBurnScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredBudgetBurnScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredBudgetBurnScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered budget burn score as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnScore {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub attempts: i64,
    pub namespace: String,
    pub audience: String,
    pub priority: i64,
    pub status: TieredBudgetBurnScoreStatus,
}

impl TieredBudgetBurnScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredBudgetBurnScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredBudgetBurnScoreStatus) -> bool {
        match self.status {
            TieredBudgetBurnScoreStatus::Created => {
                match next {
                    TieredBudgetBurnScoreStatus::Published | TieredBudgetBurnScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredBudgetBurnScoreStatus::Validated => {
                match next {
                    TieredBudgetBurnScoreStatus::Published | TieredBudgetBurnScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredBudgetBurnScoreStatus::Published => {
                return next == TieredBudgetBurnScoreStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredBudgetBurnScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered budget burn score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered budget burn score", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.score - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredBudgetBurnScore {
        TieredBudgetBurnScore {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            attempts: self.attempts,
            namespace: self.namespace.clone(),
            audience: self.audience.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a tiered budget burn score in status created.
pub fn new_tiered_budget_burn_score(code: &str, label: &str) -> TieredBudgetBurnScore {
    TieredBudgetBurnScore {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        attempts: 0,
        namespace: "".to_string(),
        audience: "".to_string(),
        priority: 0,
        status: TieredBudgetBurnScoreStatus::Created,
    }
}
