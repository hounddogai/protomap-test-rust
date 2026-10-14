use super::*;

use acme_kit::text;

/// Where a health check rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl HealthCheckRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for HealthCheckRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRule {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub port: i64,
    pub title: String,
    pub pinned: bool,
    pub updated_at: i64,
    pub threshold: f64,
    pub status: HealthCheckRuleStatus,
}

impl HealthCheckRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckRuleStatus) -> bool {
        match self.status {
            HealthCheckRuleStatus::Requested => {
                return next == HealthCheckRuleStatus::Granted || next == HealthCheckRuleStatus::Revoked;
            }
            HealthCheckRuleStatus::Granted => {
                return next == HealthCheckRuleStatus::Revoked || next == HealthCheckRuleStatus::Expired;
            }
            HealthCheckRuleStatus::Revoked => {
                match next {
                    HealthCheckRuleStatus::Expired => {
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

    pub fn move_to(&mut self, next: HealthCheckRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check rule {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "health check rule", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.threshold - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckRule {
        HealthCheckRule {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            port: self.port,
            title: self.title.clone(),
            pinned: self.pinned,
            updated_at: self.updated_at,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a health check rule in status requested.
pub fn new_health_check_rule(code: &str, name: &str) -> HealthCheckRule {
    HealthCheckRule {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        port: 0,
        title: "".to_string(),
        pinned: false,
        updated_at: 0,
        threshold: 0.0,
        status: HealthCheckRuleStatus::Requested,
    }
}
