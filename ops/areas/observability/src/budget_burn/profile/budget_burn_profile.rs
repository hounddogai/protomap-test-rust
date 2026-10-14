use super::*;

use acme_kit::text;

/// Where a budget burn profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BudgetBurnProfileStatus {
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

impl std::fmt::Display for BudgetBurnProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProfile {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub minimum: f64,
    pub priority: i64,
    pub status: BudgetBurnProfileStatus,
}

impl BudgetBurnProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnProfileStatus) -> bool {
        match self.status {
            BudgetBurnProfileStatus::Open => {
                match next {
                    BudgetBurnProfileStatus::Resolved | BudgetBurnProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnProfileStatus::InReview => {
                match next {
                    BudgetBurnProfileStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnProfileStatus::Resolved => {
                return next == BudgetBurnProfileStatus::Archived;
            }
            BudgetBurnProfileStatus::Reopened => {
                match next {
                    BudgetBurnProfileStatus::Archived => {
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

    pub fn move_to(&mut self, next: BudgetBurnProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "budget burn profile", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnProfile {
        BudgetBurnProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            minimum: self.minimum,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a budget burn profile in status open.
pub fn new_budget_burn_profile(code: &str, title: &str) -> BudgetBurnProfile {
    BudgetBurnProfile {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        minimum: 0.0,
        priority: 0,
        status: BudgetBurnProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_budget_burn_profile("code-71", "title-350");
        assert_eq!(item.status, BudgetBurnProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_budget_burn_profile("code-71", "title-350");
        if item.move_to(BudgetBurnProfileStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnProfileStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_budget_burn_profile("code-71", "title-350");
        item.status = BudgetBurnProfileStatus::Archived;
        assert!(
            item.move_to(BudgetBurnProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
