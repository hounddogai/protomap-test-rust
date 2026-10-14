use super::*;

use acme_kit::text;

/// Where a fallback rollback policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRollbackPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackRollbackPolicyStatus {
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

impl std::fmt::Display for FallbackRollbackPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback rollback policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackPolicy {
    pub code: String,
    pub title: String,
    pub region: String,
    pub reviewer: String,
    pub quota_units: i64,
    pub budget: f64,
    pub phone: String,
    pub created_at: i64,
    pub timezone: String,
    pub status: FallbackRollbackPolicyStatus,
}

impl FallbackRollbackPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRollbackPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRollbackPolicyStatus) -> bool {
        match self.status {
            FallbackRollbackPolicyStatus::Open => {
                return next == FallbackRollbackPolicyStatus::Reopened
                    || next == FallbackRollbackPolicyStatus::Archived;
            }
            FallbackRollbackPolicyStatus::InReview => {
                return next == FallbackRollbackPolicyStatus::Archived;
            }
            FallbackRollbackPolicyStatus::Resolved => {
                return next == FallbackRollbackPolicyStatus::Reopened
                    || next == FallbackRollbackPolicyStatus::Archived;
            }
            FallbackRollbackPolicyStatus::Reopened => {
                match next {
                    FallbackRollbackPolicyStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackRollbackPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback rollback policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback rollback policy", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.budget - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRollbackPolicy {
        FallbackRollbackPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            reviewer: self.reviewer.clone(),
            quota_units: self.quota_units,
            budget: self.budget,
            phone: self.phone.clone(),
            created_at: self.created_at,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a fallback rollback policy in status open.
pub fn new_fallback_rollback_policy(code: &str, title: &str) -> FallbackRollbackPolicy {
    FallbackRollbackPolicy {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        reviewer: "".to_string(),
        quota_units: 0,
        budget: 0.0,
        phone: "".to_string(),
        created_at: 0,
        timezone: "".to_string(),
        status: FallbackRollbackPolicyStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_rollback_policy("code-3", "title-281");
        assert_eq!(item.status, FallbackRollbackPolicyStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_fallback_rollback_policy("code-3", "title-281");
        if item.move_to(FallbackRollbackPolicyStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRollbackPolicyStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_rollback_policy("code-3", "title-281");
        item.status = FallbackRollbackPolicyStatus::Archived;
        assert!(
            item.move_to(FallbackRollbackPolicyStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
