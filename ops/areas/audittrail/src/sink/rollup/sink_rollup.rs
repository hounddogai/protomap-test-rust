use super::*;

use acme_kit::text;

/// Where a sink rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SinkRollupStatus {
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

impl std::fmt::Display for SinkRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRollup {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub duration_minutes: i64,
    pub phone: String,
    pub port: i64,
    pub scope: String,
    pub timezone: String,
    pub created_at: i64,
    pub weight: f64,
    pub rotation_days: i64,
    pub status: SinkRollupStatus,
}

impl SinkRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkRollupStatus) -> bool {
        match self.status {
            SinkRollupStatus::Planned => {
                match next {
                    SinkRollupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkRollupStatus::Scheduled => {
                return next == SinkRollupStatus::Cancelled;
            }
            SinkRollupStatus::InProgress => {
                return next == SinkRollupStatus::Completed || next == SinkRollupStatus::Cancelled;
            }
            SinkRollupStatus::Completed => {
                return next == SinkRollupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink rollup {} cannot move from {} to {}",
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
        format!("{} {}", "sink rollup", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.duration_minutes as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkRollup {
        SinkRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            duration_minutes: self.duration_minutes,
            phone: self.phone.clone(),
            port: self.port,
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            created_at: self.created_at,
            weight: self.weight,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a sink rollup in status planned.
pub fn new_sink_rollup(code: &str, title: &str) -> SinkRollup {
    SinkRollup {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        duration_minutes: 0,
        phone: "".to_string(),
        port: 0,
        scope: "".to_string(),
        timezone: "".to_string(),
        created_at: 0,
        weight: 0.0,
        rotation_days: 0,
        status: SinkRollupStatus::Planned,
    }
}
