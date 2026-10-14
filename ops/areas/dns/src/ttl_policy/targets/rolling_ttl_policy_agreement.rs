use super::*;

use acme_kit::text;

/// Where a rolling ttl policy agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTtlPolicyAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingTtlPolicyAgreementStatus {
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

impl std::fmt::Display for RollingTtlPolicyAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling ttl policy agreement as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyAgreement {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub notes: String,
    pub active: bool,
    pub status: RollingTtlPolicyAgreementStatus,
}

impl RollingTtlPolicyAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTtlPolicyAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTtlPolicyAgreementStatus) -> bool {
        match self.status {
            RollingTtlPolicyAgreementStatus::Open => {
                return next == RollingTtlPolicyAgreementStatus::Resolved;
            }
            RollingTtlPolicyAgreementStatus::InReview => {
                match next {
                    RollingTtlPolicyAgreementStatus::Reopened | RollingTtlPolicyAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTtlPolicyAgreementStatus::Resolved => {
                return next == RollingTtlPolicyAgreementStatus::Reopened
                    || next == RollingTtlPolicyAgreementStatus::Archived;
            }
            RollingTtlPolicyAgreementStatus::Reopened => {
                match next {
                    RollingTtlPolicyAgreementStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingTtlPolicyAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling ttl policy agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling ttl policy agreement", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> RollingTtlPolicyAgreement {
        RollingTtlPolicyAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            notes: self.notes.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling ttl policy agreement in status open.
pub fn new_rolling_ttl_policy_agreement(code: &str, name: &str) -> RollingTtlPolicyAgreement {
    RollingTtlPolicyAgreement {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        notes: "".to_string(),
        active: false,
        status: RollingTtlPolicyAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_ttl_policy_agreement("code-914", "name-791");
        assert_eq!(item.status, RollingTtlPolicyAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rolling_ttl_policy_agreement("code-914", "name-791");
        if item.move_to(RollingTtlPolicyAgreementStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingTtlPolicyAgreementStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_ttl_policy_agreement("code-914", "name-791");
        item.status = RollingTtlPolicyAgreementStatus::Archived;
        assert!(
            item.move_to(RollingTtlPolicyAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
