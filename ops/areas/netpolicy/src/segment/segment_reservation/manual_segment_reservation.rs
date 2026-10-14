use super::*;

use acme_kit::text;

/// Where a manual segment reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSegmentReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualSegmentReservationStatus {
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

impl std::fmt::Display for ManualSegmentReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual segment reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSegmentReservation {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub active: bool,
    pub key_id: String,
    pub quantity: i64,
    pub status: ManualSegmentReservationStatus,
}

impl ManualSegmentReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSegmentReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSegmentReservationStatus) -> bool {
        match self.status {
            ManualSegmentReservationStatus::Planned => {
                match next {
                    ManualSegmentReservationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSegmentReservationStatus::Scheduled => {
                match next {
                    ManualSegmentReservationStatus::InProgress | ManualSegmentReservationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSegmentReservationStatus::InProgress => {
                return next == ManualSegmentReservationStatus::Cancelled;
            }
            ManualSegmentReservationStatus::Completed => {
                return next == ManualSegmentReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualSegmentReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual segment reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ManualSegmentReservation {
        ManualSegmentReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            active: self.active,
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual segment reservation in status planned.
pub fn new_manual_segment_reservation(code: &str, title: &str) -> ManualSegmentReservation {
    ManualSegmentReservation {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        active: false,
        key_id: "".to_string(),
        quantity: 0,
        status: ManualSegmentReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_segment_reservation("code-69", "title-559");
        assert_eq!(item.status, ManualSegmentReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_manual_segment_reservation("code-69", "title-559");
        if item.move_to(ManualSegmentReservationStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSegmentReservationStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_segment_reservation("code-69", "title-559");
        item.status = ManualSegmentReservationStatus::Cancelled;
        assert!(
            item.move_to(ManualSegmentReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
