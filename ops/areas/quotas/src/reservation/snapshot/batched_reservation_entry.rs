use super::*;

use acme_kit::text;

/// Where a batched reservation entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedReservationEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedReservationEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedReservationEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched reservation entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationEntry {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub ceiling: f64,
    pub maximum: f64,
    pub notes: String,
    pub status: BatchedReservationEntryStatus,
}

impl BatchedReservationEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedReservationEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedReservationEntryStatus) -> bool {
        match self.status {
            BatchedReservationEntryStatus::Idle => {
                return next == BatchedReservationEntryStatus::Draining;
            }
            BatchedReservationEntryStatus::Warming => {
                return next == BatchedReservationEntryStatus::Ready || next == BatchedReservationEntryStatus::Draining;
            }
            BatchedReservationEntryStatus::Ready => {
                match next {
                    BatchedReservationEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedReservationEntryStatus::Draining => {
                match next {
                    BatchedReservationEntryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedReservationEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched reservation entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched reservation entry", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ceiling + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedReservationEntry {
        BatchedReservationEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            ceiling: self.ceiling,
            maximum: self.maximum,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched reservation entry in status idle.
pub fn new_batched_reservation_entry(code: &str, name: &str) -> BatchedReservationEntry {
    BatchedReservationEntry {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        ceiling: 0.0,
        maximum: 0.0,
        notes: "".to_string(),
        status: BatchedReservationEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_reservation_entry("code-220", "name-299");
        assert_eq!(item.status, BatchedReservationEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_batched_reservation_entry("code-220", "name-299");
        if item.move_to(BatchedReservationEntryStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedReservationEntryStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_reservation_entry("code-220", "name-299");
        item.status = BatchedReservationEntryStatus::Stopped;
        assert!(
            item.move_to(BatchedReservationEntryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
