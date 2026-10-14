use super::*;

use acme_kit::text;

/// Where a seasonal health check segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalHealthCheckSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalHealthCheckSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalHealthCheckSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal health check segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckSegment {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub cluster: String,
    pub locale: String,
    pub minimum: f64,
    pub limit: i64,
    pub timezone: String,
    pub quantity: i64,
    pub status: SeasonalHealthCheckSegmentStatus,
}

impl SeasonalHealthCheckSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalHealthCheckSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalHealthCheckSegmentStatus) -> bool {
        match self.status {
            SeasonalHealthCheckSegmentStatus::Created => {
                match next {
                    SeasonalHealthCheckSegmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalHealthCheckSegmentStatus::Validated => {
                match next {
                    SeasonalHealthCheckSegmentStatus::Published | SeasonalHealthCheckSegmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalHealthCheckSegmentStatus::Published => {
                return next == SeasonalHealthCheckSegmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalHealthCheckSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal health check segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.burst as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalHealthCheckSegment {
        SeasonalHealthCheckSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            cluster: self.cluster.clone(),
            locale: self.locale.clone(),
            minimum: self.minimum,
            limit: self.limit,
            timezone: self.timezone.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal health check segment in status created.
pub fn new_seasonal_health_check_segment(code: &str, label: &str) -> SeasonalHealthCheckSegment {
    SeasonalHealthCheckSegment {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        cluster: "".to_string(),
        locale: "".to_string(),
        minimum: 0.0,
        limit: 0,
        timezone: "".to_string(),
        quantity: 0,
        status: SeasonalHealthCheckSegmentStatus::Created,
    }
}
