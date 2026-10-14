use super::*;

use acme_kit::text;

/// Where a pending freeze reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFreezeReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingFreezeReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingFreezeReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending freeze reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeReservation {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub rotation_days: i64,
    pub batch_size: i64,
    pub threshold: f64,
    pub active: bool,
    pub ratio: f64,
    pub pinned: bool,
    pub status: PendingFreezeReservationStatus,
}

impl PendingFreezeReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFreezeReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFreezeReservationStatus) -> bool {
        match self.status {
            PendingFreezeReservationStatus::Proposed => {
                return next == PendingFreezeReservationStatus::Withdrawn;
            }
            PendingFreezeReservationStatus::Approved => {
                return next == PendingFreezeReservationStatus::Withdrawn;
            }
            PendingFreezeReservationStatus::Rejected => {
                match next {
                    PendingFreezeReservationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingFreezeReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending freeze reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.threshold - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFreezeReservation {
        PendingFreezeReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            rotation_days: self.rotation_days,
            batch_size: self.batch_size,
            threshold: self.threshold,
            active: self.active,
            ratio: self.ratio,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending freeze reservation in status proposed.
pub fn new_pending_freeze_reservation(code: &str, name: &str) -> PendingFreezeReservation {
    PendingFreezeReservation {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        rotation_days: 0,
        batch_size: 0,
        threshold: 0.0,
        active: false,
        ratio: 0.0,
        pinned: false,
        status: PendingFreezeReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_freeze_reservation("code-870", "name-18");
        assert_eq!(item.status, PendingFreezeReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_pending_freeze_reservation("code-870", "name-18");
        if item.move_to(PendingFreezeReservationStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFreezeReservationStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_freeze_reservation("code-870", "name-18");
        item.status = PendingFreezeReservationStatus::Withdrawn;
        assert!(
            item.move_to(PendingFreezeReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
