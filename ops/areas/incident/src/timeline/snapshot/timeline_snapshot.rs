use super::*;

use acme_kit::text;

/// Where a timeline snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TimelineSnapshotStatus {
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

impl std::fmt::Display for TimelineSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSnapshot {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub maximum: f64,
    pub namespace: String,
    pub category: String,
    pub margin: f64,
    pub active: bool,
    pub updated_at: i64,
    pub status: TimelineSnapshotStatus,
}

impl TimelineSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineSnapshotStatus) -> bool {
        match self.status {
            TimelineSnapshotStatus::Planned => {
                match next {
                    TimelineSnapshotStatus::Scheduled | TimelineSnapshotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineSnapshotStatus::Scheduled => {
                match next {
                    TimelineSnapshotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineSnapshotStatus::InProgress => {
                return next == TimelineSnapshotStatus::Completed || next == TimelineSnapshotStatus::Cancelled;
            }
            TimelineSnapshotStatus::Completed => {
                return next == TimelineSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "timeline snapshot", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        (self.margin * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineSnapshot {
        TimelineSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            maximum: self.maximum,
            namespace: self.namespace.clone(),
            category: self.category.clone(),
            margin: self.margin,
            active: self.active,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a timeline snapshot in status planned.
pub fn new_timeline_snapshot(code: &str, title: &str) -> TimelineSnapshot {
    TimelineSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        maximum: 0.0,
        namespace: "".to_string(),
        category: "".to_string(),
        margin: 0.0,
        active: false,
        updated_at: 0,
        status: TimelineSnapshotStatus::Planned,
    }
}
