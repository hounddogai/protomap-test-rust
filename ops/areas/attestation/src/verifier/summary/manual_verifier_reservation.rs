use super::*;

use acme_kit::text;

/// Where a manual verifier reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualVerifierReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualVerifierReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ManualVerifierReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual verifier reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierReservation {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub segment: String,
    pub batch_size: i64,
    pub status: ManualVerifierReservationStatus,
}

impl ManualVerifierReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualVerifierReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualVerifierReservationStatus) -> bool {
        match self.status {
            ManualVerifierReservationStatus::New => {
                return next == ManualVerifierReservationStatus::Queued
                    || next == ManualVerifierReservationStatus::Done;
            }
            ManualVerifierReservationStatus::Queued => {
                return next == ManualVerifierReservationStatus::Running
                    || next == ManualVerifierReservationStatus::Done;
            }
            ManualVerifierReservationStatus::Running => {
                return next == ManualVerifierReservationStatus::Done
                    || next == ManualVerifierReservationStatus::Failed;
            }
            ManualVerifierReservationStatus::Done => {
                return next == ManualVerifierReservationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualVerifierReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual verifier reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual verifier reservation", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.batch_size as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualVerifierReservation {
        ManualVerifierReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            segment: self.segment.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual verifier reservation in status new.
pub fn new_manual_verifier_reservation(code: &str, title: &str) -> ManualVerifierReservation {
    ManualVerifierReservation {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        segment: "".to_string(),
        batch_size: 0,
        status: ManualVerifierReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_verifier_reservation("code-645", "title-110");
        assert_eq!(item.status, ManualVerifierReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_manual_verifier_reservation("code-645", "title-110");
        if item.move_to(ManualVerifierReservationStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualVerifierReservationStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_verifier_reservation("code-645", "title-110");
        item.status = ManualVerifierReservationStatus::Failed;
        assert!(
            item.move_to(ManualVerifierReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
