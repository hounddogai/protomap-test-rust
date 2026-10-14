use super::*;

use acme_kit::text;

/// Where a pending reservation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingReservationSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingReservationSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingReservationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending reservation slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingReservationSlot {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub cluster: String,
    pub ends_at: i64,
    pub status: PendingReservationSlotStatus,
}

impl PendingReservationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingReservationSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingReservationSlotStatus) -> bool {
        match self.status {
            PendingReservationSlotStatus::Proposed => {
                return next == PendingReservationSlotStatus::Approved
                    || next == PendingReservationSlotStatus::Rejected;
            }
            PendingReservationSlotStatus::Approved => {
                match next {
                    PendingReservationSlotStatus::Rejected | PendingReservationSlotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingReservationSlotStatus::Rejected => {
                match next {
                    PendingReservationSlotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingReservationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending reservation slot {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> PendingReservationSlot {
        PendingReservationSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            cluster: self.cluster.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a pending reservation slot in status proposed.
pub fn new_pending_reservation_slot(code: &str, title: &str) -> PendingReservationSlot {
    PendingReservationSlot {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        cluster: "".to_string(),
        ends_at: 0,
        status: PendingReservationSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_reservation_slot("code-994", "title-370");
        assert_eq!(item.status, PendingReservationSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_pending_reservation_slot("code-994", "title-370");
        if item.move_to(PendingReservationSlotStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingReservationSlotStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_reservation_slot("code-994", "title-370");
        item.status = PendingReservationSlotStatus::Withdrawn;
        assert!(
            item.move_to(PendingReservationSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
