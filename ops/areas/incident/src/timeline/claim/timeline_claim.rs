use super::*;

use acme_kit::text;

/// Where a timeline claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TimelineClaimStatus {
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

impl std::fmt::Display for TimelineClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClaim {
    pub code: String,
    pub name: String,
    pub source: String,
    pub minimum: f64,
    pub version: i64,
    pub quantity: i64,
    pub phone: String,
    pub budget: f64,
    pub status: TimelineClaimStatus,
}

impl TimelineClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineClaimStatus) -> bool {
        match self.status {
            TimelineClaimStatus::Planned => {
                match next {
                    TimelineClaimStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineClaimStatus::Scheduled => {
                return next == TimelineClaimStatus::Completed;
            }
            TimelineClaimStatus::InProgress => {
                match next {
                    TimelineClaimStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineClaimStatus::Completed => {
                match next {
                    TimelineClaimStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TimelineClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.quantity as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineClaim {
        TimelineClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            minimum: self.minimum,
            version: self.version,
            quantity: self.quantity,
            phone: self.phone.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a timeline claim in status planned.
pub fn new_timeline_claim(code: &str, name: &str) -> TimelineClaim {
    TimelineClaim {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        minimum: 0.0,
        version: 0,
        quantity: 0,
        phone: "".to_string(),
        budget: 0.0,
        status: TimelineClaimStatus::Planned,
    }
}
