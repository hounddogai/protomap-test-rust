use super::*;

use acme_kit::text;

/// Where a policy notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PolicyNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PolicyNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyNotice {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub limit: i64,
    pub region: String,
    pub approved: bool,
    pub budget: f64,
    pub key_id: String,
    pub tenant_id: String,
    pub ttl_seconds: i64,
    pub status: PolicyNoticeStatus,
}

impl PolicyNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyNoticeStatus) -> bool {
        match self.status {
            PolicyNoticeStatus::Requested => {
                match next {
                    PolicyNoticeStatus::Granted | PolicyNoticeStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyNoticeStatus::Granted => {
                match next {
                    PolicyNoticeStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyNoticeStatus::Revoked => {
                match next {
                    PolicyNoticeStatus::Expired => {
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

    pub fn move_to(&mut self, next: PolicyNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.retries as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyNotice {
        PolicyNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            limit: self.limit,
            region: self.region.clone(),
            approved: self.approved,
            budget: self.budget,
            key_id: self.key_id.clone(),
            tenant_id: self.tenant_id.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a policy notice in status requested.
pub fn new_policy_notice(code: &str, title: &str) -> PolicyNotice {
    PolicyNotice {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        limit: 0,
        region: "".to_string(),
        approved: false,
        budget: 0.0,
        key_id: "".to_string(),
        tenant_id: "".to_string(),
        ttl_seconds: 0,
        status: PolicyNoticeStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_policy_notice("code-670", "title-769");
        assert_eq!(item.status, PolicyNoticeStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_policy_notice("code-670", "title-769");
        if item.move_to(PolicyNoticeStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyNoticeStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_policy_notice("code-670", "title-769");
        item.status = PolicyNoticeStatus::Expired;
        assert!(
            item.move_to(PolicyNoticeStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
