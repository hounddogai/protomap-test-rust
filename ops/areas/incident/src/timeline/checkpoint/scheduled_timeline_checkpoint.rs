use super::*;

use acme_kit::text;

/// Where a scheduled timeline checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTimelineCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledTimelineCheckpointStatus {
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

impl std::fmt::Display for ScheduledTimelineCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled timeline checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineCheckpoint {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub quota_units: i64,
    pub active: bool,
    pub limit: i64,
    pub status: ScheduledTimelineCheckpointStatus,
}

impl ScheduledTimelineCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTimelineCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTimelineCheckpointStatus) -> bool {
        match self.status {
            ScheduledTimelineCheckpointStatus::Open => {
                match next {
                    ScheduledTimelineCheckpointStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTimelineCheckpointStatus::InReview => {
                return next == ScheduledTimelineCheckpointStatus::Resolved
                    || next == ScheduledTimelineCheckpointStatus::Reopened;
            }
            ScheduledTimelineCheckpointStatus::Resolved => {
                return next == ScheduledTimelineCheckpointStatus::Reopened
                    || next == ScheduledTimelineCheckpointStatus::Archived;
            }
            ScheduledTimelineCheckpointStatus::Reopened => {
                match next {
                    ScheduledTimelineCheckpointStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledTimelineCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled timeline checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.limit as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTimelineCheckpoint {
        ScheduledTimelineCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            quota_units: self.quota_units,
            active: self.active,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a scheduled timeline checkpoint in status open.
pub fn new_scheduled_timeline_checkpoint(code: &str, title: &str) -> ScheduledTimelineCheckpoint {
    ScheduledTimelineCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        quota_units: 0,
        active: false,
        limit: 0,
        status: ScheduledTimelineCheckpointStatus::Open,
    }
}
