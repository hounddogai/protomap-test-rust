use super::*;

use acme_kit::text;

/// Where a scheduled suspension allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSuspensionAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledSuspensionAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ScheduledSuspensionAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled suspension allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionAllocation {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub floor_value: f64,
    pub ends_at: i64,
    pub status: ScheduledSuspensionAllocationStatus,
}

impl ScheduledSuspensionAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSuspensionAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSuspensionAllocationStatus) -> bool {
        match self.status {
            ScheduledSuspensionAllocationStatus::New => {
                return next == ScheduledSuspensionAllocationStatus::Done;
            }
            ScheduledSuspensionAllocationStatus::Queued => {
                return next == ScheduledSuspensionAllocationStatus::Done;
            }
            ScheduledSuspensionAllocationStatus::Running => {
                return next == ScheduledSuspensionAllocationStatus::Done
                    || next == ScheduledSuspensionAllocationStatus::Failed;
            }
            ScheduledSuspensionAllocationStatus::Done => {
                return next == ScheduledSuspensionAllocationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledSuspensionAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled suspension allocation {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ends_at as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSuspensionAllocation {
        ScheduledSuspensionAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            floor_value: self.floor_value,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled suspension allocation in status new.
pub fn new_scheduled_suspension_allocation(code: &str, name: &str) -> ScheduledSuspensionAllocation {
    ScheduledSuspensionAllocation {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        floor_value: 0.0,
        ends_at: 0,
        status: ScheduledSuspensionAllocationStatus::New,
    }
}
