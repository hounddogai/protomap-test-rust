use super::*;

use acme_kit::text;

/// Where a peering budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PeeringBudgetStatus {
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

impl std::fmt::Display for PeeringBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a peering budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringBudget {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub visible: bool,
    pub contact_email: String,
    pub grace_days: i64,
    pub status: PeeringBudgetStatus,
}

impl PeeringBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringBudgetStatus) -> bool {
        match self.status {
            PeeringBudgetStatus::Open => {
                return next == PeeringBudgetStatus::Archived;
            }
            PeeringBudgetStatus::InReview => {
                return next == PeeringBudgetStatus::Reopened || next == PeeringBudgetStatus::Archived;
            }
            PeeringBudgetStatus::Resolved => {
                return next == PeeringBudgetStatus::Reopened;
            }
            PeeringBudgetStatus::Reopened => {
                match next {
                    PeeringBudgetStatus::Archived => {
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

    pub fn move_to(&mut self, next: PeeringBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "peering budget", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.grace_days as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringBudget {
        PeeringBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            visible: self.visible,
            contact_email: self.contact_email.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a peering budget in status open.
pub fn new_peering_budget(code: &str, name: &str) -> PeeringBudget {
    PeeringBudget {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        visible: false,
        contact_email: "".to_string(),
        grace_days: 0,
        status: PeeringBudgetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_peering_budget("code-808", "name-767");
        assert_eq!(item.status, PeeringBudgetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_peering_budget("code-808", "name-767");
        if item.move_to(PeeringBudgetStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringBudgetStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_peering_budget("code-808", "name-767");
        item.status = PeeringBudgetStatus::Archived;
        assert!(
            item.move_to(PeeringBudgetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
