use super::*;

use acme_kit::text;

/// Where a federation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FederationBudgetStatus {
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

impl std::fmt::Display for FederationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation budget as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationBudget {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub attempts: i64,
    pub active: bool,
    pub category: String,
    pub timezone: String,
    pub status: FederationBudgetStatus,
}

impl FederationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationBudgetStatus) -> bool {
        match self.status {
            FederationBudgetStatus::Open => {
                return next == FederationBudgetStatus::InReview || next == FederationBudgetStatus::Reopened;
            }
            FederationBudgetStatus::InReview => {
                return next == FederationBudgetStatus::Reopened;
            }
            FederationBudgetStatus::Resolved => {
                return next == FederationBudgetStatus::Archived;
            }
            FederationBudgetStatus::Reopened => {
                return next == FederationBudgetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.ratio * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationBudget {
        FederationBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            attempts: self.attempts,
            active: self.active,
            category: self.category.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a federation budget in status open.
pub fn new_federation_budget(code: &str, name: &str) -> FederationBudget {
    FederationBudget {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        attempts: 0,
        active: false,
        category: "".to_string(),
        timezone: "".to_string(),
        status: FederationBudgetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_federation_budget("code-108", "name-303");
        assert_eq!(item.status, FederationBudgetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_federation_budget("code-108", "name-303");
        if item.move_to(FederationBudgetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationBudgetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_federation_budget("code-108", "name-303");
        item.status = FederationBudgetStatus::Archived;
        assert!(
            item.move_to(FederationBudgetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
