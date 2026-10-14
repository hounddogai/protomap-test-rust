use super::*;

use acme_kit::text;

/// Where a reservation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationMilestoneStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReservationMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReservationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a reservation milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMilestone {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub locale: String,
    pub ratio: f64,
    pub quota_units: i64,
    pub status: ReservationMilestoneStatus,
}

impl ReservationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationMilestoneStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationMilestoneStatus) -> bool {
        match self.status {
            ReservationMilestoneStatus::Proposed => {
                match next {
                    ReservationMilestoneStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationMilestoneStatus::Approved => {
                return next == ReservationMilestoneStatus::Rejected || next == ReservationMilestoneStatus::Withdrawn;
            }
            ReservationMilestoneStatus::Rejected => {
                return next == ReservationMilestoneStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ratio - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationMilestone {
        ReservationMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            locale: self.locale.clone(),
            ratio: self.ratio,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a reservation milestone in status proposed.
pub fn new_reservation_milestone(code: &str, name: &str) -> ReservationMilestone {
    ReservationMilestone {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        locale: "".to_string(),
        ratio: 0.0,
        quota_units: 0,
        status: ReservationMilestoneStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_reservation_milestone("code-939", "name-802");
        assert_eq!(item.status, ReservationMilestoneStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_reservation_milestone("code-939", "name-802");
        if item.move_to(ReservationMilestoneStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationMilestoneStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_reservation_milestone("code-939", "name-802");
        item.status = ReservationMilestoneStatus::Withdrawn;
        assert!(
            item.move_to(ReservationMilestoneStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
