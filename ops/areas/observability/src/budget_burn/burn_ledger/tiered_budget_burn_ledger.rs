use super::*;

use acme_kit::text;

/// Where a tiered budget burn ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredBudgetBurnLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredBudgetBurnLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TieredBudgetBurnLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered budget burn ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnLedger {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub locale: String,
    pub ratio: f64,
    pub score: f64,
    pub limit: i64,
    pub margin: f64,
    pub rotation_days: i64,
    pub duration_minutes: i64,
    pub status: TieredBudgetBurnLedgerStatus,
}

impl TieredBudgetBurnLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredBudgetBurnLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredBudgetBurnLedgerStatus) -> bool {
        match self.status {
            TieredBudgetBurnLedgerStatus::New => {
                match next {
                    TieredBudgetBurnLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredBudgetBurnLedgerStatus::Queued => {
                return next == TieredBudgetBurnLedgerStatus::Running || next == TieredBudgetBurnLedgerStatus::Done;
            }
            TieredBudgetBurnLedgerStatus::Running => {
                match next {
                    TieredBudgetBurnLedgerStatus::Done | TieredBudgetBurnLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredBudgetBurnLedgerStatus::Done => {
                match next {
                    TieredBudgetBurnLedgerStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredBudgetBurnLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered budget burn ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ratio - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredBudgetBurnLedger {
        TieredBudgetBurnLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            locale: self.locale.clone(),
            ratio: self.ratio,
            score: self.score,
            limit: self.limit,
            margin: self.margin,
            rotation_days: self.rotation_days,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tiered budget burn ledger in status new.
pub fn new_tiered_budget_burn_ledger(code: &str, title: &str) -> TieredBudgetBurnLedger {
    TieredBudgetBurnLedger {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        locale: "".to_string(),
        ratio: 0.0,
        score: 0.0,
        limit: 0,
        margin: 0.0,
        rotation_days: 0,
        duration_minutes: 0,
        status: TieredBudgetBurnLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_budget_burn_ledger("code-511", "title-499");
        assert_eq!(item.status, TieredBudgetBurnLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_tiered_budget_burn_ledger("code-511", "title-499");
        if item.move_to(TieredBudgetBurnLedgerStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredBudgetBurnLedgerStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_budget_burn_ledger("code-511", "title-499");
        item.status = TieredBudgetBurnLedgerStatus::Failed;
        assert!(
            item.move_to(TieredBudgetBurnLedgerStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
