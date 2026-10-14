use super::*;

use acme_kit::text;

/// Where a manual ttl policy override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTtlPolicyOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualTtlPolicyOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualTtlPolicyOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual ttl policy override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyOverride {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub batch_size: i64,
    pub maximum: f64,
    pub expires_at: i64,
    pub status: ManualTtlPolicyOverrideStatus,
}

impl ManualTtlPolicyOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTtlPolicyOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTtlPolicyOverrideStatus) -> bool {
        match self.status {
            ManualTtlPolicyOverrideStatus::Requested => {
                return next == ManualTtlPolicyOverrideStatus::Revoked
                    || next == ManualTtlPolicyOverrideStatus::Expired;
            }
            ManualTtlPolicyOverrideStatus::Granted => {
                match next {
                    ManualTtlPolicyOverrideStatus::Revoked | ManualTtlPolicyOverrideStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTtlPolicyOverrideStatus::Revoked => {
                match next {
                    ManualTtlPolicyOverrideStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualTtlPolicyOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual ttl policy override {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.window_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTtlPolicyOverride {
        ManualTtlPolicyOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            batch_size: self.batch_size,
            maximum: self.maximum,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual ttl policy override in status requested.
pub fn new_manual_ttl_policy_override(code: &str, name: &str) -> ManualTtlPolicyOverride {
    ManualTtlPolicyOverride {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        batch_size: 0,
        maximum: 0.0,
        expires_at: 0,
        status: ManualTtlPolicyOverrideStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_ttl_policy_override("code-118", "name-982");
        assert_eq!(item.status, ManualTtlPolicyOverrideStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_manual_ttl_policy_override("code-118", "name-982");
        if item.move_to(ManualTtlPolicyOverrideStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTtlPolicyOverrideStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_ttl_policy_override("code-118", "name-982");
        item.status = ManualTtlPolicyOverrideStatus::Expired;
        assert!(
            item.move_to(ManualTtlPolicyOverrideStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
