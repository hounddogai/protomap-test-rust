use super::*;

use acme_kit::text;

/// Where a default budget burn allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultBudgetBurnAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultBudgetBurnAllocationStatus {
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

impl std::fmt::Display for DefaultBudgetBurnAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default budget burn allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnAllocation {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub score: f64,
    pub cipher: String,
    pub status: DefaultBudgetBurnAllocationStatus,
}

impl DefaultBudgetBurnAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultBudgetBurnAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultBudgetBurnAllocationStatus) -> bool {
        match self.status {
            DefaultBudgetBurnAllocationStatus::Open => {
                match next {
                    DefaultBudgetBurnAllocationStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultBudgetBurnAllocationStatus::InReview => {
                match next {
                    DefaultBudgetBurnAllocationStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultBudgetBurnAllocationStatus::Resolved => {
                return next == DefaultBudgetBurnAllocationStatus::Reopened
                    || next == DefaultBudgetBurnAllocationStatus::Archived;
            }
            DefaultBudgetBurnAllocationStatus::Reopened => {
                return next == DefaultBudgetBurnAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultBudgetBurnAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default budget burn allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        (self.cost * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultBudgetBurnAllocation {
        DefaultBudgetBurnAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            score: self.score,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default budget burn allocation in status open.
pub fn new_default_budget_burn_allocation(code: &str, title: &str) -> DefaultBudgetBurnAllocation {
    DefaultBudgetBurnAllocation {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        score: 0.0,
        cipher: "".to_string(),
        status: DefaultBudgetBurnAllocationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_budget_burn_allocation("code-517", "title-144");
        assert_eq!(item.status, DefaultBudgetBurnAllocationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_default_budget_burn_allocation("code-517", "title-144");
        if item.move_to(DefaultBudgetBurnAllocationStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultBudgetBurnAllocationStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_budget_burn_allocation("code-517", "title-144");
        item.status = DefaultBudgetBurnAllocationStatus::Archived;
        assert!(
            item.move_to(DefaultBudgetBurnAllocationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
