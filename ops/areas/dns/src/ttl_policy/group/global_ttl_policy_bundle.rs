use super::*;

use acme_kit::text;

/// Where a global ttl policy bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTtlPolicyBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalTtlPolicyBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalTtlPolicyBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global ttl policy bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyBundle {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub description: String,
    pub ratio: f64,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub capacity: i64,
    pub phone: String,
    pub updated_at: i64,
    pub status: GlobalTtlPolicyBundleStatus,
}

impl GlobalTtlPolicyBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTtlPolicyBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTtlPolicyBundleStatus) -> bool {
        match self.status {
            GlobalTtlPolicyBundleStatus::Requested => {
                return next == GlobalTtlPolicyBundleStatus::Revoked;
            }
            GlobalTtlPolicyBundleStatus::Granted => {
                match next {
                    GlobalTtlPolicyBundleStatus::Revoked | GlobalTtlPolicyBundleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTtlPolicyBundleStatus::Revoked => {
                return next == GlobalTtlPolicyBundleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalTtlPolicyBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global ttl policy bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.updated_at as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTtlPolicyBundle {
        GlobalTtlPolicyBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            description: self.description.clone(),
            ratio: self.ratio,
            pinned: self.pinned,
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            capacity: self.capacity,
            phone: self.phone.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global ttl policy bundle in status requested.
pub fn new_global_ttl_policy_bundle(code: &str, label: &str) -> GlobalTtlPolicyBundle {
    GlobalTtlPolicyBundle {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        description: "".to_string(),
        ratio: 0.0,
        pinned: false,
        tags: Vec::new(),
        grace_days: 0,
        capacity: 0,
        phone: "".to_string(),
        updated_at: 0,
        status: GlobalTtlPolicyBundleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_ttl_policy_bundle("code-402", "label-519");
        assert_eq!(item.status, GlobalTtlPolicyBundleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_global_ttl_policy_bundle("code-402", "label-519");
        if item.move_to(GlobalTtlPolicyBundleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalTtlPolicyBundleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_ttl_policy_bundle("code-402", "label-519");
        item.status = GlobalTtlPolicyBundleStatus::Expired;
        assert!(
            item.move_to(GlobalTtlPolicyBundleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
