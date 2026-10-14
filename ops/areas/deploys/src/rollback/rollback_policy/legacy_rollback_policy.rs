use super::*;

use acme_kit::text;

/// Where a legacy rollback policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRollbackPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyRollbackPolicyStatus {
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

impl std::fmt::Display for LegacyRollbackPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy rollback policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackPolicy {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub locale: String,
    pub revision: i64,
    pub status: LegacyRollbackPolicyStatus,
}

impl LegacyRollbackPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRollbackPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRollbackPolicyStatus) -> bool {
        match self.status {
            LegacyRollbackPolicyStatus::Open => {
                match next {
                    LegacyRollbackPolicyStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRollbackPolicyStatus::InReview => {
                match next {
                    LegacyRollbackPolicyStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRollbackPolicyStatus::Resolved => {
                match next {
                    LegacyRollbackPolicyStatus::Reopened | LegacyRollbackPolicyStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRollbackPolicyStatus::Reopened => {
                match next {
                    LegacyRollbackPolicyStatus::Archived => {
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

    pub fn move_to(&mut self, next: LegacyRollbackPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollback policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> LegacyRollbackPolicy {
        LegacyRollbackPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            locale: self.locale.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy rollback policy in status open.
pub fn new_legacy_rollback_policy(code: &str, title: &str) -> LegacyRollbackPolicy {
    LegacyRollbackPolicy {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        locale: "".to_string(),
        revision: 0,
        status: LegacyRollbackPolicyStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_rollback_policy("code-725", "title-944");
        assert_eq!(item.status, LegacyRollbackPolicyStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_legacy_rollback_policy("code-725", "title-944");
        if item.move_to(LegacyRollbackPolicyStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRollbackPolicyStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_rollback_policy("code-725", "title-944");
        item.status = LegacyRollbackPolicyStatus::Archived;
        assert!(
            item.move_to(LegacyRollbackPolicyStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
