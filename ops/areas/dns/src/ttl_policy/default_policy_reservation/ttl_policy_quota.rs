use super::*;

use acme_kit::text;

/// Where a ttl policy quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TtlPolicyQuotaStatus {
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

impl std::fmt::Display for TtlPolicyQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyQuota {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub archived: bool,
    pub batch_size: i64,
    pub channel: String,
    pub host: String,
    pub segment: String,
    pub cluster: String,
    pub grace_days: i64,
    pub status: TtlPolicyQuotaStatus,
}

impl TtlPolicyQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyQuotaStatus) -> bool {
        match self.status {
            TtlPolicyQuotaStatus::Open => {
                match next {
                    TtlPolicyQuotaStatus::InReview | TtlPolicyQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyQuotaStatus::InReview => {
                return next == TtlPolicyQuotaStatus::Resolved;
            }
            TtlPolicyQuotaStatus::Resolved => {
                match next {
                    TtlPolicyQuotaStatus::Reopened | TtlPolicyQuotaStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyQuotaStatus::Reopened => {
                return next == TtlPolicyQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.floor_value + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyQuota {
        TtlPolicyQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            archived: self.archived,
            batch_size: self.batch_size,
            channel: self.channel.clone(),
            host: self.host.clone(),
            segment: self.segment.clone(),
            cluster: self.cluster.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a ttl policy quota in status open.
pub fn new_ttl_policy_quota(code: &str, name: &str) -> TtlPolicyQuota {
    TtlPolicyQuota {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        archived: false,
        batch_size: 0,
        channel: "".to_string(),
        host: "".to_string(),
        segment: "".to_string(),
        cluster: "".to_string(),
        grace_days: 0,
        status: TtlPolicyQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_ttl_policy_quota("code-618", "name-972");
        assert_eq!(item.status, TtlPolicyQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_ttl_policy_quota("code-618", "name-972");
        if item.move_to(TtlPolicyQuotaStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyQuotaStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_ttl_policy_quota("code-618", "name-972");
        item.status = TtlPolicyQuotaStatus::Archived;
        assert!(
            item.move_to(TtlPolicyQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
