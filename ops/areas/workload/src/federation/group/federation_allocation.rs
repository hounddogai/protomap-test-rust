use super::*;

use acme_kit::text;

/// Where a federation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FederationAllocationStatus {
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

impl std::fmt::Display for FederationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationAllocation {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub segment: String,
    pub version: i64,
    pub batch_size: i64,
    pub status: FederationAllocationStatus,
}

impl FederationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationAllocationStatus) -> bool {
        match self.status {
            FederationAllocationStatus::Planned => {
                return next == FederationAllocationStatus::Scheduled;
            }
            FederationAllocationStatus::Scheduled => {
                return next == FederationAllocationStatus::InProgress;
            }
            FederationAllocationStatus::InProgress => {
                match next {
                    FederationAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationAllocationStatus::Completed => {
                return next == FederationAllocationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation allocation {} cannot move from {} to {}",
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.version as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationAllocation {
        FederationAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            segment: self.segment.clone(),
            version: self.version,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a federation allocation in status planned.
pub fn new_federation_allocation(code: &str, name: &str) -> FederationAllocation {
    FederationAllocation {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        segment: "".to_string(),
        version: 0,
        batch_size: 0,
        status: FederationAllocationStatus::Planned,
    }
}
