use super::*;

use acme_kit::text;

/// Where a budget burn ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BudgetBurnLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BudgetBurnLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a budget burn ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnLedger {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub limit: i64,
    pub name: String,
    pub locale: String,
    pub ceiling: f64,
    pub starts_at: i64,
    pub ttl_seconds: i64,
    pub status: BudgetBurnLedgerStatus,
}

impl BudgetBurnLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnLedgerStatus) -> bool {
        match self.status {
            BudgetBurnLedgerStatus::Created => {
                return next == BudgetBurnLedgerStatus::Validated;
            }
            BudgetBurnLedgerStatus::Validated => {
                match next {
                    BudgetBurnLedgerStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnLedgerStatus::Published => {
                match next {
                    BudgetBurnLedgerStatus::Retired => {
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

    pub fn move_to(&mut self, next: BudgetBurnLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.starts_at as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnLedger {
        BudgetBurnLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            limit: self.limit,
            name: self.name.clone(),
            locale: self.locale.clone(),
            ceiling: self.ceiling,
            starts_at: self.starts_at,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a budget burn ledger in status created.
pub fn new_budget_burn_ledger(code: &str, label: &str) -> BudgetBurnLedger {
    BudgetBurnLedger {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        limit: 0,
        name: "".to_string(),
        locale: "".to_string(),
        ceiling: 0.0,
        starts_at: 0,
        ttl_seconds: 0,
        status: BudgetBurnLedgerStatus::Created,
    }
}
