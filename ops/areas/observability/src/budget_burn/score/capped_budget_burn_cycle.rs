use super::*;

use acme_kit::text;

/// Where a capped budget burn cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedBudgetBurnCycleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedBudgetBurnCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for CappedBudgetBurnCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped budget burn cycle as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnCycle {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub margin: f64,
    pub amount: f64,
    pub ends_at: i64,
    pub reviewer: String,
    pub source: String,
    pub cost: f64,
    pub title: String,
    pub status: CappedBudgetBurnCycleStatus,
}

impl CappedBudgetBurnCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedBudgetBurnCycleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedBudgetBurnCycleStatus) -> bool {
        match self.status {
            CappedBudgetBurnCycleStatus::Open => {
                return next == CappedBudgetBurnCycleStatus::Archived;
            }
            CappedBudgetBurnCycleStatus::InReview => {
                return next == CappedBudgetBurnCycleStatus::Resolved || next == CappedBudgetBurnCycleStatus::Reopened;
            }
            CappedBudgetBurnCycleStatus::Resolved => {
                match next {
                    CappedBudgetBurnCycleStatus::Reopened | CappedBudgetBurnCycleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnCycleStatus::Reopened => {
                match next {
                    CappedBudgetBurnCycleStatus::Archived => {
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

    pub fn move_to(&mut self, next: CappedBudgetBurnCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped budget burn cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.attempts as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedBudgetBurnCycle {
        CappedBudgetBurnCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            margin: self.margin,
            amount: self.amount,
            ends_at: self.ends_at,
            reviewer: self.reviewer.clone(),
            source: self.source.clone(),
            cost: self.cost,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped budget burn cycle in status open.
pub fn new_capped_budget_burn_cycle(code: &str, label: &str) -> CappedBudgetBurnCycle {
    CappedBudgetBurnCycle {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        margin: 0.0,
        amount: 0.0,
        ends_at: 0,
        reviewer: "".to_string(),
        source: "".to_string(),
        cost: 0.0,
        title: "".to_string(),
        status: CappedBudgetBurnCycleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_budget_burn_cycle("code-529", "label-826");
        assert_eq!(item.status, CappedBudgetBurnCycleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_capped_budget_burn_cycle("code-529", "label-826");
        if item.move_to(CappedBudgetBurnCycleStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedBudgetBurnCycleStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_budget_burn_cycle("code-529", "label-826");
        item.status = CappedBudgetBurnCycleStatus::Archived;
        assert!(
            item.move_to(CappedBudgetBurnCycleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
