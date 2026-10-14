use super::*;

use acme_kit::text;

/// Where a rollout allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RolloutAllocationStatus {
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

impl std::fmt::Display for RolloutAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollout allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAllocation {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub checksum: String,
    pub created_at: i64,
    pub quota_units: i64,
    pub category: String,
    pub status: RolloutAllocationStatus,
}

impl RolloutAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutAllocationStatus) -> bool {
        match self.status {
            RolloutAllocationStatus::New => {
                return next == RolloutAllocationStatus::Running;
            }
            RolloutAllocationStatus::Queued => {
                return next == RolloutAllocationStatus::Running || next == RolloutAllocationStatus::Done;
            }
            RolloutAllocationStatus::Running => {
                return next == RolloutAllocationStatus::Done || next == RolloutAllocationStatus::Failed;
            }
            RolloutAllocationStatus::Done => {
                match next {
                    RolloutAllocationStatus::Failed => {
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

    pub fn move_to(&mut self, next: RolloutAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.quota_units as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutAllocation {
        RolloutAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            quota_units: self.quota_units,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rollout allocation in status new.
pub fn new_rollout_allocation(code: &str, name: &str) -> RolloutAllocation {
    RolloutAllocation {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        checksum: "".to_string(),
        created_at: 0,
        quota_units: 0,
        category: "".to_string(),
        status: RolloutAllocationStatus::New,
    }
}
