use super::*;

use acme_kit::text;

/// Where a freeze rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeRuleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FreezeRuleStatus {
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

impl std::fmt::Display for FreezeRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRule {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub revision: i64,
    pub tenant_id: String,
    pub pinned: bool,
    pub grace_days: i64,
    pub currency: String,
    pub sequence: i64,
    pub status: FreezeRuleStatus,
}

impl FreezeRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeRuleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeRuleStatus) -> bool {
        match self.status {
            FreezeRuleStatus::Open => {
                return next == FreezeRuleStatus::InReview || next == FreezeRuleStatus::Archived;
            }
            FreezeRuleStatus::InReview => {
                return next == FreezeRuleStatus::Reopened || next == FreezeRuleStatus::Archived;
            }
            FreezeRuleStatus::Resolved => {
                return next == FreezeRuleStatus::Archived;
            }
            FreezeRuleStatus::Reopened => {
                match next {
                    FreezeRuleStatus::Archived => {
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

    pub fn move_to(&mut self, next: FreezeRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.sequence as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeRule {
        FreezeRule {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            revision: self.revision,
            tenant_id: self.tenant_id.clone(),
            pinned: self.pinned,
            grace_days: self.grace_days,
            currency: self.currency.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a freeze rule in status open.
pub fn new_freeze_rule(code: &str, name: &str) -> FreezeRule {
    FreezeRule {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        revision: 0,
        tenant_id: "".to_string(),
        pinned: false,
        grace_days: 0,
        currency: "".to_string(),
        sequence: 0,
        status: FreezeRuleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_freeze_rule("code-224", "name-576");
        assert_eq!(item.status, FreezeRuleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_freeze_rule("code-224", "name-576");
        if item.move_to(FreezeRuleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeRuleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_freeze_rule("code-224", "name-576");
        item.status = FreezeRuleStatus::Archived;
        assert!(item.move_to(FreezeRuleStatus::Open).is_err(), "expected move to fail");
    }
}
