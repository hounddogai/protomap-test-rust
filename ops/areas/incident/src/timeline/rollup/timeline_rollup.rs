use super::*;

use acme_kit::text;

/// Where a timeline rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TimelineRollupStatus {
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

impl std::fmt::Display for TimelineRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRollup {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub weight: f64,
    pub ends_at: i64,
    pub reviewer: String,
    pub issuer: String,
    pub sequence: i64,
    pub pinned: bool,
    pub status: TimelineRollupStatus,
}

impl TimelineRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineRollupStatus) -> bool {
        match self.status {
            TimelineRollupStatus::Planned => {
                match next {
                    TimelineRollupStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineRollupStatus::Scheduled => {
                return next == TimelineRollupStatus::Completed;
            }
            TimelineRollupStatus::InProgress => {
                match next {
                    TimelineRollupStatus::Completed | TimelineRollupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineRollupStatus::Completed => {
                return next == TimelineRollupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline rollup {} cannot move from {} to {}",
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
        format!("{} {}", "timeline rollup", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.sequence as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineRollup {
        TimelineRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            weight: self.weight,
            ends_at: self.ends_at,
            reviewer: self.reviewer.clone(),
            issuer: self.issuer.clone(),
            sequence: self.sequence,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a timeline rollup in status planned.
pub fn new_timeline_rollup(code: &str, label: &str) -> TimelineRollup {
    TimelineRollup {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        weight: 0.0,
        ends_at: 0,
        reviewer: "".to_string(),
        issuer: "".to_string(),
        sequence: 0,
        pinned: false,
        status: TimelineRollupStatus::Planned,
    }
}
