use super::*;

use acme_kit::text;

/// Where a health check estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl HealthCheckEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for HealthCheckEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEstimate {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub unit: String,
    pub name: String,
    pub currency: String,
    pub region: String,
    pub key_id: String,
    pub maximum: f64,
    pub status: HealthCheckEstimateStatus,
}

impl HealthCheckEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckEstimateStatus) -> bool {
        match self.status {
            HealthCheckEstimateStatus::Planned => {
                match next {
                    HealthCheckEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckEstimateStatus::Scheduled => {
                return next == HealthCheckEstimateStatus::Completed || next == HealthCheckEstimateStatus::Cancelled;
            }
            HealthCheckEstimateStatus::InProgress => {
                return next == HealthCheckEstimateStatus::Cancelled;
            }
            HealthCheckEstimateStatus::Completed => {
                match next {
                    HealthCheckEstimateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: HealthCheckEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "health check estimate", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ends_at as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckEstimate {
        HealthCheckEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            unit: self.unit.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            region: self.region.clone(),
            key_id: self.key_id.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a health check estimate in status planned.
pub fn new_health_check_estimate(code: &str, label: &str) -> HealthCheckEstimate {
    HealthCheckEstimate {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        unit: "".to_string(),
        name: "".to_string(),
        currency: "".to_string(),
        region: "".to_string(),
        key_id: "".to_string(),
        maximum: 0.0,
        status: HealthCheckEstimateStatus::Planned,
    }
}
