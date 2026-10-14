use super::*;

use acme_kit::text;

/// Where a approval rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ApprovalRuleStatus {
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

impl std::fmt::Display for ApprovalRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One approval rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRule {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub issuer: String,
    pub retries: i64,
    pub summary: String,
    pub scope: String,
    pub status: ApprovalRuleStatus,
}

impl ApprovalRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalRuleStatus) -> bool {
        match self.status {
            ApprovalRuleStatus::Open => {
                return next == ApprovalRuleStatus::Resolved || next == ApprovalRuleStatus::Archived;
            }
            ApprovalRuleStatus::InReview => {
                return next == ApprovalRuleStatus::Resolved || next == ApprovalRuleStatus::Reopened;
            }
            ApprovalRuleStatus::Resolved => {
                return next == ApprovalRuleStatus::Reopened;
            }
            ApprovalRuleStatus::Reopened => {
                match next {
                    ApprovalRuleStatus::Archived => {
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

    pub fn move_to(&mut self, next: ApprovalRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "approval rule", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.retries as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalRule {
        ApprovalRule {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            issuer: self.issuer.clone(),
            retries: self.retries,
            summary: self.summary.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a approval rule in status open.
pub fn new_approval_rule(code: &str, title: &str) -> ApprovalRule {
    ApprovalRule {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        issuer: "".to_string(),
        retries: 0,
        summary: "".to_string(),
        scope: "".to_string(),
        status: ApprovalRuleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_approval_rule("code-578", "title-382");
        assert_eq!(item.status, ApprovalRuleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_approval_rule("code-578", "title-382");
        if item.move_to(ApprovalRuleStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalRuleStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_approval_rule("code-578", "title-382");
        item.status = ApprovalRuleStatus::Archived;
        assert!(item.move_to(ApprovalRuleStatus::Open).is_err(), "expected move to fail");
    }
}
