use super::*;

use acme_kit::text;

/// Where a flow log reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FlowLogReservationStatus {
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

impl std::fmt::Display for FlowLogReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a flow log reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogReservation {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub ttl_seconds: i64,
    pub fingerprint: String,
    pub sequence: i64,
    pub retries: i64,
    pub floor_value: f64,
    pub weight: f64,
    pub status: FlowLogReservationStatus,
}

impl FlowLogReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogReservationStatus) -> bool {
        match self.status {
            FlowLogReservationStatus::New => {
                match next {
                    FlowLogReservationStatus::Queued | FlowLogReservationStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogReservationStatus::Queued => {
                return next == FlowLogReservationStatus::Running || next == FlowLogReservationStatus::Failed;
            }
            FlowLogReservationStatus::Running => {
                return next == FlowLogReservationStatus::Done;
            }
            FlowLogReservationStatus::Done => {
                return next == FlowLogReservationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log reservation", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.sequence as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogReservation {
        FlowLogReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            ttl_seconds: self.ttl_seconds,
            fingerprint: self.fingerprint.clone(),
            sequence: self.sequence,
            retries: self.retries,
            floor_value: self.floor_value,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a flow log reservation in status new.
pub fn new_flow_log_reservation(code: &str, title: &str) -> FlowLogReservation {
    FlowLogReservation {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        ttl_seconds: 0,
        fingerprint: "".to_string(),
        sequence: 0,
        retries: 0,
        floor_value: 0.0,
        weight: 0.0,
        status: FlowLogReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_flow_log_reservation("code-862", "title-901");
        assert_eq!(item.status, FlowLogReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_flow_log_reservation("code-862", "title-901");
        if item.move_to(FlowLogReservationStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogReservationStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_flow_log_reservation("code-862", "title-901");
        item.status = FlowLogReservationStatus::Failed;
        assert!(
            item.move_to(FlowLogReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
