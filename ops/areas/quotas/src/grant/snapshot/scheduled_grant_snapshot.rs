use super::*;

use acme_kit::text;

/// Where a scheduled grant snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledGrantSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledGrantSnapshotStatus {
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

impl std::fmt::Display for ScheduledGrantSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled grant snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantSnapshot {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub threshold: f64,
    pub quota_units: i64,
    pub channel: String,
    pub status: ScheduledGrantSnapshotStatus,
}

impl ScheduledGrantSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledGrantSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledGrantSnapshotStatus) -> bool {
        match self.status {
            ScheduledGrantSnapshotStatus::Planned => {
                match next {
                    ScheduledGrantSnapshotStatus::Scheduled | ScheduledGrantSnapshotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledGrantSnapshotStatus::Scheduled => {
                return next == ScheduledGrantSnapshotStatus::InProgress;
            }
            ScheduledGrantSnapshotStatus::InProgress => {
                return next == ScheduledGrantSnapshotStatus::Completed;
            }
            ScheduledGrantSnapshotStatus::Completed => {
                match next {
                    ScheduledGrantSnapshotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ScheduledGrantSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled grant snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled grant snapshot", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.threshold + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledGrantSnapshot {
        ScheduledGrantSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            threshold: self.threshold,
            quota_units: self.quota_units,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled grant snapshot in status planned.
pub fn new_scheduled_grant_snapshot(code: &str, label: &str) -> ScheduledGrantSnapshot {
    ScheduledGrantSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        threshold: 0.0,
        quota_units: 0,
        channel: "".to_string(),
        status: ScheduledGrantSnapshotStatus::Planned,
    }
}
