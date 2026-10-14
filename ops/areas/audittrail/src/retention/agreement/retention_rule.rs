use super::*;

use acme_kit::text;

/// Where a retention rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RetentionRuleStatus {
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

impl std::fmt::Display for RetentionRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRule {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub summary: String,
    pub tenant_id: String,
    pub approved: bool,
    pub contact_email: String,
    pub status: RetentionRuleStatus,
}

impl RetentionRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionRuleStatus) -> bool {
        match self.status {
            RetentionRuleStatus::Open => {
                match next {
                    RetentionRuleStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionRuleStatus::InReview => {
                match next {
                    RetentionRuleStatus::Resolved | RetentionRuleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionRuleStatus::Resolved => {
                return next == RetentionRuleStatus::Reopened;
            }
            RetentionRuleStatus::Reopened => {
                return next == RetentionRuleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> RetentionRule {
        RetentionRule {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            summary: self.summary.clone(),
            tenant_id: self.tenant_id.clone(),
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a retention rule in status open.
pub fn new_retention_rule(code: &str, name: &str) -> RetentionRule {
    RetentionRule {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        summary: "".to_string(),
        tenant_id: "".to_string(),
        approved: false,
        contact_email: "".to_string(),
        status: RetentionRuleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_retention_rule("code-85", "name-848");
        assert_eq!(item.status, RetentionRuleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_retention_rule("code-85", "name-848");
        if item.move_to(RetentionRuleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionRuleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_retention_rule("code-85", "name-848");
        item.status = RetentionRuleStatus::Archived;
        assert!(
            item.move_to(RetentionRuleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
