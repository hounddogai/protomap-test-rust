use super::*;

use acme_kit::text;

/// Where a primary reservation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryReservationScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryReservationScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PrimaryReservationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary reservation schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSchedule {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub grace_days: i64,
    pub issuer: String,
    pub region: String,
    pub notes: String,
    pub window_days: i64,
    pub pinned: bool,
    pub segment: String,
    pub status: PrimaryReservationScheduleStatus,
}

impl PrimaryReservationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryReservationScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryReservationScheduleStatus) -> bool {
        match self.status {
            PrimaryReservationScheduleStatus::Draft => {
                return next == PrimaryReservationScheduleStatus::Active
                    || next == PrimaryReservationScheduleStatus::Suspended;
            }
            PrimaryReservationScheduleStatus::Pending => {
                return next == PrimaryReservationScheduleStatus::Closed;
            }
            PrimaryReservationScheduleStatus::Active => {
                match next {
                    PrimaryReservationScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationScheduleStatus::Suspended => {
                return next == PrimaryReservationScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryReservationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary reservation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.grace_days as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryReservationSchedule {
        PrimaryReservationSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            grace_days: self.grace_days,
            issuer: self.issuer.clone(),
            region: self.region.clone(),
            notes: self.notes.clone(),
            window_days: self.window_days,
            pinned: self.pinned,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary reservation schedule in status draft.
pub fn new_primary_reservation_schedule(code: &str, label: &str) -> PrimaryReservationSchedule {
    PrimaryReservationSchedule {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        grace_days: 0,
        issuer: "".to_string(),
        region: "".to_string(),
        notes: "".to_string(),
        window_days: 0,
        pinned: false,
        segment: "".to_string(),
        status: PrimaryReservationScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_reservation_schedule("code-397", "label-418");
        assert_eq!(item.status, PrimaryReservationScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_primary_reservation_schedule("code-397", "label-418");
        if item.move_to(PrimaryReservationScheduleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryReservationScheduleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_reservation_schedule("code-397", "label-418");
        item.status = PrimaryReservationScheduleStatus::Closed;
        assert!(
            item.move_to(PrimaryReservationScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
