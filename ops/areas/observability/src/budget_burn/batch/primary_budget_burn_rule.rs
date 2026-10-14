use super::*;

use acme_kit::text;

/// Where a primary budget burn rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryBudgetBurnRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryBudgetBurnRuleStatus {
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

impl std::fmt::Display for PrimaryBudgetBurnRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary budget burn rule as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnRule {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub description: String,
    pub active: bool,
    pub rate: f64,
    pub weight: f64,
    pub locale: String,
    pub version: i64,
    pub status: PrimaryBudgetBurnRuleStatus,
}

impl PrimaryBudgetBurnRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryBudgetBurnRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryBudgetBurnRuleStatus) -> bool {
        match self.status {
            PrimaryBudgetBurnRuleStatus::Open => {
                return next == PrimaryBudgetBurnRuleStatus::InReview;
            }
            PrimaryBudgetBurnRuleStatus::InReview => {
                match next {
                    PrimaryBudgetBurnRuleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBudgetBurnRuleStatus::Resolved => {
                match next {
                    PrimaryBudgetBurnRuleStatus::Reopened | PrimaryBudgetBurnRuleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBudgetBurnRuleStatus::Reopened => {
                match next {
                    PrimaryBudgetBurnRuleStatus::Archived => {
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

    pub fn move_to(&mut self, next: PrimaryBudgetBurnRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary budget burn rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rate - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryBudgetBurnRule {
        PrimaryBudgetBurnRule {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            description: self.description.clone(),
            active: self.active,
            rate: self.rate,
            weight: self.weight,
            locale: self.locale.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary budget burn rule in status open.
pub fn new_primary_budget_burn_rule(code: &str, title: &str) -> PrimaryBudgetBurnRule {
    PrimaryBudgetBurnRule {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        description: "".to_string(),
        active: false,
        rate: 0.0,
        weight: 0.0,
        locale: "".to_string(),
        version: 0,
        status: PrimaryBudgetBurnRuleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_budget_burn_rule("code-728", "title-643");
        assert_eq!(item.status, PrimaryBudgetBurnRuleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_primary_budget_burn_rule("code-728", "title-643");
        if item.move_to(PrimaryBudgetBurnRuleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryBudgetBurnRuleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_budget_burn_rule("code-728", "title-643");
        item.status = PrimaryBudgetBurnRuleStatus::Archived;
        assert!(
            item.move_to(PrimaryBudgetBurnRuleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
