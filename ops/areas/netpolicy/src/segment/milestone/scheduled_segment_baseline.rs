use super::*;

use acme_kit::text;

/// Where a scheduled segment baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSegmentBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledSegmentBaselineStatus {
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

impl std::fmt::Display for ScheduledSegmentBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled segment baseline as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentBaseline {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub capacity: i64,
    pub amount: f64,
    pub phone: String,
    pub owner_id: String,
    pub ratio: f64,
    pub status: ScheduledSegmentBaselineStatus,
}

impl ScheduledSegmentBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSegmentBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSegmentBaselineStatus) -> bool {
        match self.status {
            ScheduledSegmentBaselineStatus::Planned => {
                match next {
                    ScheduledSegmentBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSegmentBaselineStatus::Scheduled => {
                match next {
                    ScheduledSegmentBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSegmentBaselineStatus::InProgress => {
                match next {
                    ScheduledSegmentBaselineStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSegmentBaselineStatus::Completed => {
                match next {
                    ScheduledSegmentBaselineStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ScheduledSegmentBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled segment baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.amount - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSegmentBaseline {
        ScheduledSegmentBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            capacity: self.capacity,
            amount: self.amount,
            phone: self.phone.clone(),
            owner_id: self.owner_id.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled segment baseline in status planned.
pub fn new_scheduled_segment_baseline(code: &str, name: &str) -> ScheduledSegmentBaseline {
    ScheduledSegmentBaseline {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        capacity: 0,
        amount: 0.0,
        phone: "".to_string(),
        owner_id: "".to_string(),
        ratio: 0.0,
        status: ScheduledSegmentBaselineStatus::Planned,
    }
}
