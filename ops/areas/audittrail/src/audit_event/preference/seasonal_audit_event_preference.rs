use super::*;

use acme_kit::text;

/// Where a seasonal audit event preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalAuditEventPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalAuditEventPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SeasonalAuditEventPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal audit event preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAuditEventPreference {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub locale: String,
    pub minimum: f64,
    pub ttl_seconds: i64,
    pub principal: String,
    pub rotation_days: i64,
    pub category: String,
    pub amount: f64,
    pub priority: i64,
    pub status: SeasonalAuditEventPreferenceStatus,
}

impl SeasonalAuditEventPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalAuditEventPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalAuditEventPreferenceStatus) -> bool {
        match self.status {
            SeasonalAuditEventPreferenceStatus::Open => {
                match next {
                    SeasonalAuditEventPreferenceStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAuditEventPreferenceStatus::InReview => {
                return next == SeasonalAuditEventPreferenceStatus::Reopened;
            }
            SeasonalAuditEventPreferenceStatus::Resolved => {
                match next {
                    SeasonalAuditEventPreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAuditEventPreferenceStatus::Reopened => {
                match next {
                    SeasonalAuditEventPreferenceStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalAuditEventPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal audit event preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.priority as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalAuditEventPreference {
        SeasonalAuditEventPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            locale: self.locale.clone(),
            minimum: self.minimum,
            ttl_seconds: self.ttl_seconds,
            principal: self.principal.clone(),
            rotation_days: self.rotation_days,
            category: self.category.clone(),
            amount: self.amount,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal audit event preference in status open.
pub fn new_seasonal_audit_event_preference(code: &str, name: &str) -> SeasonalAuditEventPreference {
    SeasonalAuditEventPreference {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        locale: "".to_string(),
        minimum: 0.0,
        ttl_seconds: 0,
        principal: "".to_string(),
        rotation_days: 0,
        category: "".to_string(),
        amount: 0.0,
        priority: 0,
        status: SeasonalAuditEventPreferenceStatus::Open,
    }
}
