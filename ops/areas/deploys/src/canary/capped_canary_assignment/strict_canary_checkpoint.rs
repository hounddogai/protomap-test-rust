use super::*;

use acme_kit::text;

/// Where a strict canary checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictCanaryCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictCanaryCheckpointStatus {
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

impl std::fmt::Display for StrictCanaryCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict canary checkpoint as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryCheckpoint {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub limit: i64,
    pub ceiling: f64,
    pub duration_minutes: i64,
    pub score: f64,
    pub status: StrictCanaryCheckpointStatus,
}

impl StrictCanaryCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictCanaryCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictCanaryCheckpointStatus) -> bool {
        match self.status {
            StrictCanaryCheckpointStatus::Planned => {
                return next == StrictCanaryCheckpointStatus::Cancelled;
            }
            StrictCanaryCheckpointStatus::Scheduled => {
                match next {
                    StrictCanaryCheckpointStatus::Completed | StrictCanaryCheckpointStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictCanaryCheckpointStatus::InProgress => {
                return next == StrictCanaryCheckpointStatus::Completed
                    || next == StrictCanaryCheckpointStatus::Cancelled;
            }
            StrictCanaryCheckpointStatus::Completed => {
                return next == StrictCanaryCheckpointStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictCanaryCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict canary checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ceiling - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictCanaryCheckpoint {
        StrictCanaryCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            limit: self.limit,
            ceiling: self.ceiling,
            duration_minutes: self.duration_minutes,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict canary checkpoint in status planned.
pub fn new_strict_canary_checkpoint(code: &str, name: &str) -> StrictCanaryCheckpoint {
    StrictCanaryCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        limit: 0,
        ceiling: 0.0,
        duration_minutes: 0,
        score: 0.0,
        status: StrictCanaryCheckpointStatus::Planned,
    }
}
