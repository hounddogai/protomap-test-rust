use super::*;

use acme_kit::text;

/// Where a global reservation snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReservationSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalReservationSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalReservationSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global reservation snapshot as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationSnapshot {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub reviewer: String,
    pub active: bool,
    pub segment: String,
    pub discount: f64,
    pub contact_email: String,
    pub status: GlobalReservationSnapshotStatus,
}

impl GlobalReservationSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReservationSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReservationSnapshotStatus) -> bool {
        match self.status {
            GlobalReservationSnapshotStatus::Requested => {
                return next == GlobalReservationSnapshotStatus::Granted
                    || next == GlobalReservationSnapshotStatus::Expired;
            }
            GlobalReservationSnapshotStatus::Granted => {
                return next == GlobalReservationSnapshotStatus::Revoked
                    || next == GlobalReservationSnapshotStatus::Expired;
            }
            GlobalReservationSnapshotStatus::Revoked => {
                return next == GlobalReservationSnapshotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalReservationSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global reservation snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global reservation snapshot", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.discount * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReservationSnapshot {
        GlobalReservationSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            reviewer: self.reviewer.clone(),
            active: self.active,
            segment: self.segment.clone(),
            discount: self.discount,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a global reservation snapshot in status requested.
pub fn new_global_reservation_snapshot(code: &str, label: &str) -> GlobalReservationSnapshot {
    GlobalReservationSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        reviewer: "".to_string(),
        active: false,
        segment: "".to_string(),
        discount: 0.0,
        contact_email: "".to_string(),
        status: GlobalReservationSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_reservation_snapshot("code-912", "label-965");
        assert_eq!(item.status, GlobalReservationSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_global_reservation_snapshot("code-912", "label-965");
        if item.move_to(GlobalReservationSnapshotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalReservationSnapshotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_reservation_snapshot("code-912", "label-965");
        item.status = GlobalReservationSnapshotStatus::Expired;
        assert!(
            item.move_to(GlobalReservationSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
