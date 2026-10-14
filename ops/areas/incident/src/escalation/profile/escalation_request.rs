use super::*;

use acme_kit::text;

/// Where a escalation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationRequestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EscalationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EscalationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One escalation request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationRequest {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub unit: String,
    pub currency: String,
    pub ratio: f64,
    pub window_days: i64,
    pub port: i64,
    pub starts_at: i64,
    pub limit: i64,
    pub status: EscalationRequestStatus,
}

impl EscalationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationRequestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationRequestStatus) -> bool {
        match self.status {
            EscalationRequestStatus::Created => {
                match next {
                    EscalationRequestStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationRequestStatus::Validated => {
                match next {
                    EscalationRequestStatus::Published | EscalationRequestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationRequestStatus::Published => {
                return next == EscalationRequestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "escalation request", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.window_days as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationRequest {
        EscalationRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            unit: self.unit.clone(),
            currency: self.currency.clone(),
            ratio: self.ratio,
            window_days: self.window_days,
            port: self.port,
            starts_at: self.starts_at,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a escalation request in status created.
pub fn new_escalation_request(code: &str, title: &str) -> EscalationRequest {
    EscalationRequest {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        unit: "".to_string(),
        currency: "".to_string(),
        ratio: 0.0,
        window_days: 0,
        port: 0,
        starts_at: 0,
        limit: 0,
        status: EscalationRequestStatus::Created,
    }
}
