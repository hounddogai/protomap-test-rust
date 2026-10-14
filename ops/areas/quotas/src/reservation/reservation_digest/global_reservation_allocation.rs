use super::*;

use acme_kit::text;

/// Where a global reservation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReservationAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalReservationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for GlobalReservationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global reservation allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationAllocation {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub score: f64,
    pub rotation_days: i64,
    pub status: GlobalReservationAllocationStatus,
}

impl GlobalReservationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReservationAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReservationAllocationStatus) -> bool {
        match self.status {
            GlobalReservationAllocationStatus::Open => {
                match next {
                    GlobalReservationAllocationStatus::InReview | GlobalReservationAllocationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReservationAllocationStatus::InReview => {
                match next {
                    GlobalReservationAllocationStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReservationAllocationStatus::Resolved => {
                return next == GlobalReservationAllocationStatus::Reopened;
            }
            GlobalReservationAllocationStatus::Reopened => {
                return next == GlobalReservationAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalReservationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global reservation allocation {} cannot move from {} to {}",
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

    pub fn projected_volume(&self) -> f64 {
        (self.score * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReservationAllocation {
        GlobalReservationAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            score: self.score,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global reservation allocation in status open.
pub fn new_global_reservation_allocation(code: &str, name: &str) -> GlobalReservationAllocation {
    GlobalReservationAllocation {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        score: 0.0,
        rotation_days: 0,
        status: GlobalReservationAllocationStatus::Open,
    }
}
