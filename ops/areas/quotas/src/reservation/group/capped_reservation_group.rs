use super::*;

use acme_kit::text;

/// Where a capped reservation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedReservationGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedReservationGroupStatus {
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

impl std::fmt::Display for CappedReservationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped reservation group as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationGroup {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub segment: String,
    pub phone: String,
    pub amount: f64,
    pub status: CappedReservationGroupStatus,
}

impl CappedReservationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedReservationGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedReservationGroupStatus) -> bool {
        match self.status {
            CappedReservationGroupStatus::Planned => {
                match next {
                    CappedReservationGroupStatus::InProgress | CappedReservationGroupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedReservationGroupStatus::Scheduled => {
                return next == CappedReservationGroupStatus::Cancelled;
            }
            CappedReservationGroupStatus::InProgress => {
                return next == CappedReservationGroupStatus::Completed;
            }
            CappedReservationGroupStatus::Completed => {
                match next {
                    CappedReservationGroupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CappedReservationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped reservation group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> CappedReservationGroup {
        CappedReservationGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            segment: self.segment.clone(),
            phone: self.phone.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a capped reservation group in status planned.
pub fn new_capped_reservation_group(code: &str, name: &str) -> CappedReservationGroup {
    CappedReservationGroup {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        segment: "".to_string(),
        phone: "".to_string(),
        amount: 0.0,
        status: CappedReservationGroupStatus::Planned,
    }
}
