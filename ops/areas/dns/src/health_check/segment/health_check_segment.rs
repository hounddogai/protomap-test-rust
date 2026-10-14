use super::*;

use acme_kit::text;

/// Where a health check segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl HealthCheckSegmentStatus {
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

impl std::fmt::Display for HealthCheckSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSegment {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub ends_at: i64,
    pub country: String,
    pub cipher: String,
    pub retries: i64,
    pub ceiling: f64,
    pub budget: f64,
    pub status: HealthCheckSegmentStatus,
}

impl HealthCheckSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckSegmentStatus) -> bool {
        match self.status {
            HealthCheckSegmentStatus::Planned => {
                return next == HealthCheckSegmentStatus::Scheduled || next == HealthCheckSegmentStatus::Completed;
            }
            HealthCheckSegmentStatus::Scheduled => {
                match next {
                    HealthCheckSegmentStatus::InProgress | HealthCheckSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckSegmentStatus::InProgress => {
                match next {
                    HealthCheckSegmentStatus::Completed | HealthCheckSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckSegmentStatus::Completed => {
                return next == HealthCheckSegmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check segment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckSegment {
        HealthCheckSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            ends_at: self.ends_at,
            country: self.country.clone(),
            cipher: self.cipher.clone(),
            retries: self.retries,
            ceiling: self.ceiling,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a health check segment in status planned.
pub fn new_health_check_segment(code: &str, name: &str) -> HealthCheckSegment {
    HealthCheckSegment {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        ends_at: 0,
        country: "".to_string(),
        cipher: "".to_string(),
        retries: 0,
        ceiling: 0.0,
        budget: 0.0,
        status: HealthCheckSegmentStatus::Planned,
    }
}
