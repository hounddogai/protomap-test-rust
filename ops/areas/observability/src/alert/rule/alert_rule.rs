use super::*;

use acme_kit::text;

/// Where a alert rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AlertRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AlertRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertRule {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub maximum: f64,
    pub window_days: i64,
    pub retries: i64,
    pub updated_at: i64,
    pub rotation_days: i64,
    pub external_ref: String,
    pub key_id: String,
    pub status: AlertRuleStatus,
}

impl AlertRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertRuleStatus) -> bool {
        match self.status {
            AlertRuleStatus::Created => {
                return next == AlertRuleStatus::Validated;
            }
            AlertRuleStatus::Validated => {
                match next {
                    AlertRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertRuleStatus::Published => {
                return next == AlertRuleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert rule {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.retries as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertRule {
        AlertRule {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            maximum: self.maximum,
            window_days: self.window_days,
            retries: self.retries,
            updated_at: self.updated_at,
            rotation_days: self.rotation_days,
            external_ref: self.external_ref.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a alert rule in status created.
pub fn new_alert_rule(code: &str, title: &str) -> AlertRule {
    AlertRule {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        maximum: 0.0,
        window_days: 0,
        retries: 0,
        updated_at: 0,
        rotation_days: 0,
        external_ref: "".to_string(),
        key_id: "".to_string(),
        status: AlertRuleStatus::Created,
    }
}
