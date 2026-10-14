use super::*;

use acme_kit::text;

/// Where a lenient segment reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSegmentReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientSegmentReservationStatus {
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

impl std::fmt::Display for LenientSegmentReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient segment reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentReservation {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub version: i64,
    pub namespace: String,
    pub updated_at: i64,
    pub visible: bool,
    pub scope: String,
    pub notes: String,
    pub fingerprint: String,
    pub status: LenientSegmentReservationStatus,
}

impl LenientSegmentReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSegmentReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSegmentReservationStatus) -> bool {
        match self.status {
            LenientSegmentReservationStatus::Planned => {
                return next == LenientSegmentReservationStatus::InProgress
                    || next == LenientSegmentReservationStatus::Cancelled;
            }
            LenientSegmentReservationStatus::Scheduled => {
                return next == LenientSegmentReservationStatus::Completed
                    || next == LenientSegmentReservationStatus::Cancelled;
            }
            LenientSegmentReservationStatus::InProgress => {
                return next == LenientSegmentReservationStatus::Cancelled;
            }
            LenientSegmentReservationStatus::Completed => {
                return next == LenientSegmentReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSegmentReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient segment reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient segment reservation", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.version as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSegmentReservation {
        LenientSegmentReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            version: self.version,
            namespace: self.namespace.clone(),
            updated_at: self.updated_at,
            visible: self.visible,
            scope: self.scope.clone(),
            notes: self.notes.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient segment reservation in status planned.
pub fn new_lenient_segment_reservation(code: &str, name: &str) -> LenientSegmentReservation {
    LenientSegmentReservation {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        version: 0,
        namespace: "".to_string(),
        updated_at: 0,
        visible: false,
        scope: "".to_string(),
        notes: "".to_string(),
        fingerprint: "".to_string(),
        status: LenientSegmentReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_segment_reservation("code-496", "name-999");
        assert_eq!(item.status, LenientSegmentReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_lenient_segment_reservation("code-496", "name-999");
        if item.move_to(LenientSegmentReservationStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSegmentReservationStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_segment_reservation("code-496", "name-999");
        item.status = LenientSegmentReservationStatus::Cancelled;
        assert!(
            item.move_to(LenientSegmentReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
