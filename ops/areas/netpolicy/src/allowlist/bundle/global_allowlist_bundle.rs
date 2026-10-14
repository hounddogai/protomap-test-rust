use super::*;

use acme_kit::text;

/// Where a global allowlist bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAllowlistBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalAllowlistBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalAllowlistBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global allowlist bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAllowlistBundle {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub grace_days: i64,
    pub window_days: i64,
    pub ttl_seconds: i64,
    pub expires_at: i64,
    pub owner_id: String,
    pub approved: bool,
    pub retries: i64,
    pub segment: String,
    pub status: GlobalAllowlistBundleStatus,
}

impl GlobalAllowlistBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAllowlistBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAllowlistBundleStatus) -> bool {
        match self.status {
            GlobalAllowlistBundleStatus::Requested => {
                return next == GlobalAllowlistBundleStatus::Granted || next == GlobalAllowlistBundleStatus::Revoked;
            }
            GlobalAllowlistBundleStatus::Granted => {
                return next == GlobalAllowlistBundleStatus::Revoked;
            }
            GlobalAllowlistBundleStatus::Revoked => {
                match next {
                    GlobalAllowlistBundleStatus::Expired => {
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

    pub fn move_to(&mut self, next: GlobalAllowlistBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global allowlist bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.grace_days as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalAllowlistBundle {
        GlobalAllowlistBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            grace_days: self.grace_days,
            window_days: self.window_days,
            ttl_seconds: self.ttl_seconds,
            expires_at: self.expires_at,
            owner_id: self.owner_id.clone(),
            approved: self.approved,
            retries: self.retries,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a global allowlist bundle in status requested.
pub fn new_global_allowlist_bundle(code: &str, name: &str) -> GlobalAllowlistBundle {
    GlobalAllowlistBundle {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        grace_days: 0,
        window_days: 0,
        ttl_seconds: 0,
        expires_at: 0,
        owner_id: "".to_string(),
        approved: false,
        retries: 0,
        segment: "".to_string(),
        status: GlobalAllowlistBundleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_allowlist_bundle("code-95", "name-42");
        assert_eq!(item.status, GlobalAllowlistBundleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_global_allowlist_bundle("code-95", "name-42");
        if item.move_to(GlobalAllowlistBundleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalAllowlistBundleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_allowlist_bundle("code-95", "name-42");
        item.status = GlobalAllowlistBundleStatus::Expired;
        assert!(
            item.move_to(GlobalAllowlistBundleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
