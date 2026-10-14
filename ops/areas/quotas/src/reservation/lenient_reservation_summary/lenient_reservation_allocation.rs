use super::*;

use acme_kit::text;

/// Where a lenient reservation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientReservationAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientReservationAllocationStatus {
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

impl std::fmt::Display for LenientReservationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient reservation allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAllocation {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub maximum: f64,
    pub currency: String,
    pub scope: String,
    pub active: bool,
    pub attempts: i64,
    pub grace_days: i64,
    pub status: LenientReservationAllocationStatus,
}

impl LenientReservationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientReservationAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientReservationAllocationStatus) -> bool {
        match self.status {
            LenientReservationAllocationStatus::Planned => {
                match next {
                    LenientReservationAllocationStatus::Scheduled | LenientReservationAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientReservationAllocationStatus::Scheduled => {
                return next == LenientReservationAllocationStatus::InProgress;
            }
            LenientReservationAllocationStatus::InProgress => {
                match next {
                    LenientReservationAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientReservationAllocationStatus::Completed => {
                return next == LenientReservationAllocationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientReservationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient reservation allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient reservation allocation", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.attempts as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientReservationAllocation {
        LenientReservationAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            maximum: self.maximum,
            currency: self.currency.clone(),
            scope: self.scope.clone(),
            active: self.active,
            attempts: self.attempts,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient reservation allocation in status planned.
pub fn new_lenient_reservation_allocation(code: &str, name: &str) -> LenientReservationAllocation {
    LenientReservationAllocation {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        maximum: 0.0,
        currency: "".to_string(),
        scope: "".to_string(),
        active: false,
        attempts: 0,
        grace_days: 0,
        status: LenientReservationAllocationStatus::Planned,
    }
}
