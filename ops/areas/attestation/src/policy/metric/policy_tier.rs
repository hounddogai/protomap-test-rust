use super::*;

use acme_kit::text;

/// Where a policy tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PolicyTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PolicyTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyTier {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub namespace: String,
    pub limit: i64,
    pub amount: f64,
    pub source: String,
    pub title: String,
    pub segment: String,
    pub label: String,
    pub status: PolicyTierStatus,
}

impl PolicyTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyTierStatus) -> bool {
        match self.status {
            PolicyTierStatus::Requested => {
                return next == PolicyTierStatus::Expired;
            }
            PolicyTierStatus::Granted => {
                return next == PolicyTierStatus::Revoked || next == PolicyTierStatus::Expired;
            }
            PolicyTierStatus::Revoked => {
                match next {
                    PolicyTierStatus::Expired => {
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

    pub fn move_to(&mut self, next: PolicyTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "policy tier", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.limit as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyTier {
        PolicyTier {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            namespace: self.namespace.clone(),
            limit: self.limit,
            amount: self.amount,
            source: self.source.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a policy tier in status requested.
pub fn new_policy_tier(code: &str, name: &str) -> PolicyTier {
    PolicyTier {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        namespace: "".to_string(),
        limit: 0,
        amount: 0.0,
        source: "".to_string(),
        title: "".to_string(),
        segment: "".to_string(),
        label: "".to_string(),
        status: PolicyTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_policy_tier("code-877", "name-222");
        assert_eq!(item.status, PolicyTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_policy_tier("code-877", "name-222");
        if item.move_to(PolicyTierStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyTierStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_policy_tier("code-877", "name-222");
        item.status = PolicyTierStatus::Expired;
        assert!(
            item.move_to(PolicyTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
