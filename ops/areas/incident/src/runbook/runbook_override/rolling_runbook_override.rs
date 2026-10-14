use super::*;

use acme_kit::text;

/// Where a rolling runbook override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRunbookOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingRunbookOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingRunbookOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling runbook override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookOverride {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub category: String,
    pub expires_at: i64,
    pub status: RollingRunbookOverrideStatus,
}

impl RollingRunbookOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRunbookOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRunbookOverrideStatus) -> bool {
        match self.status {
            RollingRunbookOverrideStatus::Requested => {
                match next {
                    RollingRunbookOverrideStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRunbookOverrideStatus::Granted => {
                return next == RollingRunbookOverrideStatus::Revoked || next == RollingRunbookOverrideStatus::Expired;
            }
            RollingRunbookOverrideStatus::Revoked => {
                return next == RollingRunbookOverrideStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingRunbookOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling runbook override {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.expires_at as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRunbookOverride {
        RollingRunbookOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling runbook override in status requested.
pub fn new_rolling_runbook_override(code: &str, label: &str) -> RollingRunbookOverride {
    RollingRunbookOverride {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        category: "".to_string(),
        expires_at: 0,
        status: RollingRunbookOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_runbook_override("code-681", "label-259");
        assert_eq!(item.status, RollingRunbookOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rolling_runbook_override("code-681", "label-259");
        if item.move_to(RollingRunbookOverrideStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRunbookOverrideStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_runbook_override("code-681", "label-259");
        item.status = RollingRunbookOverrideStatus::Expired;
        assert!(
            item.move_to(RollingRunbookOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
