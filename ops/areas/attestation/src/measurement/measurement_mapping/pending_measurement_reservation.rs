use super::*;

use acme_kit::text;

/// Where a pending measurement reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingMeasurementReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingMeasurementReservationStatus {
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

impl std::fmt::Display for PendingMeasurementReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending measurement reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementReservation {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub channel: String,
    pub tags: Vec<String>,
    pub score: f64,
    pub status: PendingMeasurementReservationStatus,
}

impl PendingMeasurementReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingMeasurementReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingMeasurementReservationStatus) -> bool {
        match self.status {
            PendingMeasurementReservationStatus::New => {
                return next == PendingMeasurementReservationStatus::Done;
            }
            PendingMeasurementReservationStatus::Queued => {
                match next {
                    PendingMeasurementReservationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingMeasurementReservationStatus::Running => {
                return next == PendingMeasurementReservationStatus::Done
                    || next == PendingMeasurementReservationStatus::Failed;
            }
            PendingMeasurementReservationStatus::Done => {
                return next == PendingMeasurementReservationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingMeasurementReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending measurement reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PendingMeasurementReservation {
        PendingMeasurementReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            tags: self.tags.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending measurement reservation in status new.
pub fn new_pending_measurement_reservation(code: &str, name: &str) -> PendingMeasurementReservation {
    PendingMeasurementReservation {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        channel: "".to_string(),
        tags: Vec::new(),
        score: 0.0,
        status: PendingMeasurementReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_measurement_reservation("code-860", "name-341");
        assert_eq!(item.status, PendingMeasurementReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_pending_measurement_reservation("code-860", "name-341");
        if item.move_to(PendingMeasurementReservationStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingMeasurementReservationStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_measurement_reservation("code-860", "name-341");
        item.status = PendingMeasurementReservationStatus::Failed;
        assert!(
            item.move_to(PendingMeasurementReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
