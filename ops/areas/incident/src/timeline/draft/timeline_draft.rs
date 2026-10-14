use super::*;

use acme_kit::text;

/// Where a timeline draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TimelineDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TimelineDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDraft {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub audience: String,
    pub duration_minutes: i64,
    pub rate: f64,
    pub ceiling: f64,
    pub batch_size: i64,
    pub key_id: String,
    pub status: TimelineDraftStatus,
}

impl TimelineDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineDraftStatus) -> bool {
        match self.status {
            TimelineDraftStatus::Proposed => {
                return next == TimelineDraftStatus::Approved || next == TimelineDraftStatus::Rejected;
            }
            TimelineDraftStatus::Approved => {
                return next == TimelineDraftStatus::Rejected || next == TimelineDraftStatus::Withdrawn;
            }
            TimelineDraftStatus::Rejected => {
                return next == TimelineDraftStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rate + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineDraft {
        TimelineDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            audience: self.audience.clone(),
            duration_minutes: self.duration_minutes,
            rate: self.rate,
            ceiling: self.ceiling,
            batch_size: self.batch_size,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a timeline draft in status proposed.
pub fn new_timeline_draft(code: &str, name: &str) -> TimelineDraft {
    TimelineDraft {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        audience: "".to_string(),
        duration_minutes: 0,
        rate: 0.0,
        ceiling: 0.0,
        batch_size: 0,
        key_id: "".to_string(),
        status: TimelineDraftStatus::Proposed,
    }
}
