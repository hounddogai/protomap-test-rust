use super::*;

use acme_kit::text;

/// Where a policy agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PolicyAgreementStatus {
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

impl std::fmt::Display for PolicyAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAgreement {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub ttl_seconds: i64,
    pub host: String,
    pub grace_days: i64,
    pub ends_at: i64,
    pub region: String,
    pub status: PolicyAgreementStatus,
}

impl PolicyAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyAgreementStatus) -> bool {
        match self.status {
            PolicyAgreementStatus::Open => {
                return next == PolicyAgreementStatus::Resolved;
            }
            PolicyAgreementStatus::InReview => {
                match next {
                    PolicyAgreementStatus::Resolved | PolicyAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyAgreementStatus::Resolved => {
                match next {
                    PolicyAgreementStatus::Reopened | PolicyAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyAgreementStatus::Reopened => {
                return next == PolicyAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ends_at as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyAgreement {
        PolicyAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            ttl_seconds: self.ttl_seconds,
            host: self.host.clone(),
            grace_days: self.grace_days,
            ends_at: self.ends_at,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a policy agreement in status open.
pub fn new_policy_agreement(code: &str, title: &str) -> PolicyAgreement {
    PolicyAgreement {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        ttl_seconds: 0,
        host: "".to_string(),
        grace_days: 0,
        ends_at: 0,
        region: "".to_string(),
        status: PolicyAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_policy_agreement("code-228", "title-640");
        assert_eq!(item.status, PolicyAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_policy_agreement("code-228", "title-640");
        if item.move_to(PolicyAgreementStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyAgreementStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_policy_agreement("code-228", "title-640");
        item.status = PolicyAgreementStatus::Archived;
        assert!(
            item.move_to(PolicyAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
