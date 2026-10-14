use super::*;

use acme_kit::text;

/// Where a adaptive delegation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveDelegationBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveDelegationBudgetStatus {
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

impl std::fmt::Display for AdaptiveDelegationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive delegation budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationBudget {
    pub code: String,
    pub label: String,
    pub region: String,
    pub grace_days: i64,
    pub ratio: f64,
    pub status: AdaptiveDelegationBudgetStatus,
}

impl AdaptiveDelegationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveDelegationBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveDelegationBudgetStatus) -> bool {
        match self.status {
            AdaptiveDelegationBudgetStatus::Open => {
                return next == AdaptiveDelegationBudgetStatus::Resolved
                    || next == AdaptiveDelegationBudgetStatus::Reopened;
            }
            AdaptiveDelegationBudgetStatus::InReview => {
                match next {
                    AdaptiveDelegationBudgetStatus::Reopened | AdaptiveDelegationBudgetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDelegationBudgetStatus::Resolved => {
                return next == AdaptiveDelegationBudgetStatus::Reopened
                    || next == AdaptiveDelegationBudgetStatus::Archived;
            }
            AdaptiveDelegationBudgetStatus::Reopened => {
                match next {
                    AdaptiveDelegationBudgetStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveDelegationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive delegation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.ratio * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveDelegationBudget {
        AdaptiveDelegationBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            grace_days: self.grace_days,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a adaptive delegation budget in status open.
pub fn new_adaptive_delegation_budget(code: &str, label: &str) -> AdaptiveDelegationBudget {
    AdaptiveDelegationBudget {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        grace_days: 0,
        ratio: 0.0,
        status: AdaptiveDelegationBudgetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_delegation_budget("code-333", "label-250");
        assert_eq!(item.status, AdaptiveDelegationBudgetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_adaptive_delegation_budget("code-333", "label-250");
        if item.move_to(AdaptiveDelegationBudgetStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveDelegationBudgetStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_delegation_budget("code-333", "label-250");
        item.status = AdaptiveDelegationBudgetStatus::Archived;
        assert!(
            item.move_to(AdaptiveDelegationBudgetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
