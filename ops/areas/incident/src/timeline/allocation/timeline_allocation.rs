use super::*;

use acme_kit::text;

/// Where a timeline allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TimelineAllocationStatus {
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

impl std::fmt::Display for TimelineAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAllocation {
    pub code: String,
    pub name: String,
    pub country: String,
    pub phone: String,
    pub grace_days: i64,
    pub priority: i64,
    pub status: TimelineAllocationStatus,
}

impl TimelineAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineAllocationStatus) -> bool {
        match self.status {
            TimelineAllocationStatus::Planned => {
                return next == TimelineAllocationStatus::Completed;
            }
            TimelineAllocationStatus::Scheduled => {
                match next {
                    TimelineAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineAllocationStatus::InProgress => {
                match next {
                    TimelineAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineAllocationStatus::Completed => {
                return next == TimelineAllocationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.grace_days as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineAllocation {
        TimelineAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            phone: self.phone.clone(),
            grace_days: self.grace_days,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a timeline allocation in status planned.
pub fn new_timeline_allocation(code: &str, name: &str) -> TimelineAllocation {
    TimelineAllocation {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        phone: "".to_string(),
        grace_days: 0,
        priority: 0,
        status: TimelineAllocationStatus::Planned,
    }
}
