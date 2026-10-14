use super::*;

use acme_kit::text;

/// Where a regional suspension budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSuspensionBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalSuspensionBudgetStatus {
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

impl std::fmt::Display for RegionalSuspensionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional suspension budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionBudget {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub quota_units: i64,
    pub score: f64,
    pub port: i64,
    pub archived: bool,
    pub status: RegionalSuspensionBudgetStatus,
}

impl RegionalSuspensionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSuspensionBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSuspensionBudgetStatus) -> bool {
        match self.status {
            RegionalSuspensionBudgetStatus::Open => {
                match next {
                    RegionalSuspensionBudgetStatus::Resolved | RegionalSuspensionBudgetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSuspensionBudgetStatus::InReview => {
                return next == RegionalSuspensionBudgetStatus::Reopened;
            }
            RegionalSuspensionBudgetStatus::Resolved => {
                return next == RegionalSuspensionBudgetStatus::Reopened
                    || next == RegionalSuspensionBudgetStatus::Archived;
            }
            RegionalSuspensionBudgetStatus::Reopened => {
                match next {
                    RegionalSuspensionBudgetStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalSuspensionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional suspension budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quota_units as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSuspensionBudget {
        RegionalSuspensionBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            quota_units: self.quota_units,
            score: self.score,
            port: self.port,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional suspension budget in status open.
pub fn new_regional_suspension_budget(code: &str, name: &str) -> RegionalSuspensionBudget {
    RegionalSuspensionBudget {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        quota_units: 0,
        score: 0.0,
        port: 0,
        archived: false,
        status: RegionalSuspensionBudgetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_suspension_budget("code-192", "name-357");
        assert_eq!(item.status, RegionalSuspensionBudgetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_regional_suspension_budget("code-192", "name-357");
        if item.move_to(RegionalSuspensionBudgetStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSuspensionBudgetStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_suspension_budget("code-192", "name-357");
        item.status = RegionalSuspensionBudgetStatus::Archived;
        assert!(
            item.move_to(RegionalSuspensionBudgetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
