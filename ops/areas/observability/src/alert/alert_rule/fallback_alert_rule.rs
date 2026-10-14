use super::*;

use acme_kit::text;

/// Where a fallback alert rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackAlertRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackAlertRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackAlertRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback alert rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRule {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub reviewer: String,
    pub updated_at: i64,
    pub channel: String,
    pub status: FallbackAlertRuleStatus,
}

impl FallbackAlertRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackAlertRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackAlertRuleStatus) -> bool {
        match self.status {
            FallbackAlertRuleStatus::Requested => {
                return next == FallbackAlertRuleStatus::Granted;
            }
            FallbackAlertRuleStatus::Granted => {
                match next {
                    FallbackAlertRuleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackAlertRuleStatus::Revoked => {
                return next == FallbackAlertRuleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackAlertRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback alert rule {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.updated_at as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackAlertRule {
        FallbackAlertRule {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            reviewer: self.reviewer.clone(),
            updated_at: self.updated_at,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a fallback alert rule in status requested.
pub fn new_fallback_alert_rule(code: &str, name: &str) -> FallbackAlertRule {
    FallbackAlertRule {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        reviewer: "".to_string(),
        updated_at: 0,
        channel: "".to_string(),
        status: FallbackAlertRuleStatus::Requested,
    }
}
