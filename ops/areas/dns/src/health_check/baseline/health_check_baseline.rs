use super::*;

use acme_kit::text;

/// Where a health check baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl HealthCheckBaselineStatus {
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

impl std::fmt::Display for HealthCheckBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBaseline {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub ttl_seconds: i64,
    pub created_at: i64,
    pub audience: String,
    pub cluster: String,
    pub status: HealthCheckBaselineStatus,
}

impl HealthCheckBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckBaselineStatus) -> bool {
        match self.status {
            HealthCheckBaselineStatus::Planned => {
                match next {
                    HealthCheckBaselineStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckBaselineStatus::Scheduled => {
                match next {
                    HealthCheckBaselineStatus::InProgress | HealthCheckBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckBaselineStatus::InProgress => {
                return next == HealthCheckBaselineStatus::Completed;
            }
            HealthCheckBaselineStatus::Completed => {
                match next {
                    HealthCheckBaselineStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: HealthCheckBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.floor_value + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckBaseline {
        HealthCheckBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            ttl_seconds: self.ttl_seconds,
            created_at: self.created_at,
            audience: self.audience.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a health check baseline in status planned.
pub fn new_health_check_baseline(code: &str, name: &str) -> HealthCheckBaseline {
    HealthCheckBaseline {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        ttl_seconds: 0,
        created_at: 0,
        audience: "".to_string(),
        cluster: "".to_string(),
        status: HealthCheckBaselineStatus::Planned,
    }
}
