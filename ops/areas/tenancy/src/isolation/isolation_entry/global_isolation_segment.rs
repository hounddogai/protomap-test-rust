use super::*;

use acme_kit::text;

/// Where a global isolation segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalIsolationSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalIsolationSegmentStatus {
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

impl std::fmt::Display for GlobalIsolationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global isolation segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationSegment {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub phone: String,
    pub weight: f64,
    pub status: GlobalIsolationSegmentStatus,
}

impl GlobalIsolationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalIsolationSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalIsolationSegmentStatus) -> bool {
        match self.status {
            GlobalIsolationSegmentStatus::Planned => {
                return next == GlobalIsolationSegmentStatus::Completed;
            }
            GlobalIsolationSegmentStatus::Scheduled => {
                match next {
                    GlobalIsolationSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIsolationSegmentStatus::InProgress => {
                return next == GlobalIsolationSegmentStatus::Completed
                    || next == GlobalIsolationSegmentStatus::Cancelled;
            }
            GlobalIsolationSegmentStatus::Completed => {
                match next {
                    GlobalIsolationSegmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalIsolationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global isolation segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.discount + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalIsolationSegment {
        GlobalIsolationSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            phone: self.phone.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global isolation segment in status planned.
pub fn new_global_isolation_segment(code: &str, label: &str) -> GlobalIsolationSegment {
    GlobalIsolationSegment {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        phone: "".to_string(),
        weight: 0.0,
        status: GlobalIsolationSegmentStatus::Planned,
    }
}
