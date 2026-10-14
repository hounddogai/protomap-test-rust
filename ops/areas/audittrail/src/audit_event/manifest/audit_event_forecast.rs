use super::*;

use acme_kit::text;

/// Where a audit event forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventForecast {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub namespace: String,
    pub limit: i64,
    pub currency: String,
    pub capacity: i64,
    pub attempts: i64,
    pub principal: String,
    pub starts_at: i64,
    pub revision: i64,
    pub status: AuditEventForecastStatus,
}

impl AuditEventForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventForecastStatus) -> bool {
        match self.status {
            AuditEventForecastStatus::Proposed => {
                match next {
                    AuditEventForecastStatus::Approved | AuditEventForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventForecastStatus::Approved => {
                return next == AuditEventForecastStatus::Rejected;
            }
            AuditEventForecastStatus::Rejected => {
                return next == AuditEventForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.attempts as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventForecast {
        AuditEventForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            namespace: self.namespace.clone(),
            limit: self.limit,
            currency: self.currency.clone(),
            capacity: self.capacity,
            attempts: self.attempts,
            principal: self.principal.clone(),
            starts_at: self.starts_at,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a audit event forecast in status proposed.
pub fn new_audit_event_forecast(code: &str, label: &str) -> AuditEventForecast {
    AuditEventForecast {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        namespace: "".to_string(),
        limit: 0,
        currency: "".to_string(),
        capacity: 0,
        attempts: 0,
        principal: "".to_string(),
        starts_at: 0,
        revision: 0,
        status: AuditEventForecastStatus::Proposed,
    }
}
