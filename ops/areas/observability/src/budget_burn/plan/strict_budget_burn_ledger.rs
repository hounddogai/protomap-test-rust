use super::*;

use acme_kit::text;

/// Where a strict budget burn ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictBudgetBurnLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictBudgetBurnLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictBudgetBurnLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict budget burn ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnLedger {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub rate: f64,
    pub external_ref: String,
    pub scope: String,
    pub source: String,
    pub batch_size: i64,
    pub principal: String,
    pub quota_units: i64,
    pub status: StrictBudgetBurnLedgerStatus,
}

impl StrictBudgetBurnLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictBudgetBurnLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictBudgetBurnLedgerStatus) -> bool {
        match self.status {
            StrictBudgetBurnLedgerStatus::Proposed => {
                return next == StrictBudgetBurnLedgerStatus::Approved
                    || next == StrictBudgetBurnLedgerStatus::Rejected;
            }
            StrictBudgetBurnLedgerStatus::Approved => {
                return next == StrictBudgetBurnLedgerStatus::Rejected;
            }
            StrictBudgetBurnLedgerStatus::Rejected => {
                match next {
                    StrictBudgetBurnLedgerStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: StrictBudgetBurnLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict budget burn ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.rate + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictBudgetBurnLedger {
        StrictBudgetBurnLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            rate: self.rate,
            external_ref: self.external_ref.clone(),
            scope: self.scope.clone(),
            source: self.source.clone(),
            batch_size: self.batch_size,
            principal: self.principal.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a strict budget burn ledger in status proposed.
pub fn new_strict_budget_burn_ledger(code: &str, label: &str) -> StrictBudgetBurnLedger {
    StrictBudgetBurnLedger {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        rate: 0.0,
        external_ref: "".to_string(),
        scope: "".to_string(),
        source: "".to_string(),
        batch_size: 0,
        principal: "".to_string(),
        quota_units: 0,
        status: StrictBudgetBurnLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_strict_budget_burn_ledger("code-143", "label-551");
        assert_eq!(item.status, StrictBudgetBurnLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_strict_budget_burn_ledger("code-143", "label-551");
        if item.move_to(StrictBudgetBurnLedgerStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictBudgetBurnLedgerStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_strict_budget_burn_ledger("code-143", "label-551");
        item.status = StrictBudgetBurnLedgerStatus::Withdrawn;
        assert!(
            item.move_to(StrictBudgetBurnLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
