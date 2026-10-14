use super::*;

use acme_kit::text;

/// Where a rolling limit override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingLimitOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingLimitOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingLimitOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling limit override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitOverride {
    pub code: String,
    pub title: String,
    pub region: String,
    pub grace_days: i64,
    pub principal: String,
    pub threshold: f64,
    pub key_id: String,
    pub namespace: String,
    pub status: RollingLimitOverrideStatus,
}

impl RollingLimitOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingLimitOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingLimitOverrideStatus) -> bool {
        match self.status {
            RollingLimitOverrideStatus::Requested => {
                match next {
                    RollingLimitOverrideStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingLimitOverrideStatus::Granted => {
                return next == RollingLimitOverrideStatus::Revoked || next == RollingLimitOverrideStatus::Expired;
            }
            RollingLimitOverrideStatus::Revoked => {
                match next {
                    RollingLimitOverrideStatus::Expired => {
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

    pub fn move_to(&mut self, next: RollingLimitOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling limit override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.threshold - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingLimitOverride {
        RollingLimitOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            grace_days: self.grace_days,
            principal: self.principal.clone(),
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rolling limit override in status requested.
pub fn new_rolling_limit_override(code: &str, title: &str) -> RollingLimitOverride {
    RollingLimitOverride {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        grace_days: 0,
        principal: "".to_string(),
        threshold: 0.0,
        key_id: "".to_string(),
        namespace: "".to_string(),
        status: RollingLimitOverrideStatus::Requested,
    }
}
