use super::*;

use acme_kit::text;

/// Where a batched binding reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedBindingReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedBindingReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedBindingReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched binding reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingReservation {
    pub code: String,
    pub label: String,
    pub title: String,
    pub quota_units: i64,
    pub reviewer: String,
    pub weight: f64,
    pub unit: String,
    pub ttl_seconds: i64,
    pub fingerprint: String,
    pub quantity: i64,
    pub amount: f64,
    pub status: BatchedBindingReservationStatus,
}

impl BatchedBindingReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedBindingReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedBindingReservationStatus) -> bool {
        match self.status {
            BatchedBindingReservationStatus::Created => {
                return next == BatchedBindingReservationStatus::Retired;
            }
            BatchedBindingReservationStatus::Validated => {
                return next == BatchedBindingReservationStatus::Retired;
            }
            BatchedBindingReservationStatus::Published => {
                match next {
                    BatchedBindingReservationStatus::Retired => {
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

    pub fn move_to(&mut self, next: BatchedBindingReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched binding reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.amount + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedBindingReservation {
        BatchedBindingReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            reviewer: self.reviewer.clone(),
            weight: self.weight,
            unit: self.unit.clone(),
            ttl_seconds: self.ttl_seconds,
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a batched binding reservation in status created.
pub fn new_batched_binding_reservation(code: &str, label: &str) -> BatchedBindingReservation {
    BatchedBindingReservation {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        quota_units: 0,
        reviewer: "".to_string(),
        weight: 0.0,
        unit: "".to_string(),
        ttl_seconds: 0,
        fingerprint: "".to_string(),
        quantity: 0,
        amount: 0.0,
        status: BatchedBindingReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_binding_reservation("code-606", "label-590");
        assert_eq!(item.status, BatchedBindingReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_batched_binding_reservation("code-606", "label-590");
        if item.move_to(BatchedBindingReservationStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedBindingReservationStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_binding_reservation("code-606", "label-590");
        item.status = BatchedBindingReservationStatus::Retired;
        assert!(
            item.move_to(BatchedBindingReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
