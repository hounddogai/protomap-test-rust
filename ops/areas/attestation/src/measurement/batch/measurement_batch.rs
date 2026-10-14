use super::*;

use acme_kit::text;

/// Where a measurement batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl MeasurementBatchStatus {
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

impl std::fmt::Display for MeasurementBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement batch as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBatch {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub currency: String,
    pub tenant_id: String,
    pub status: MeasurementBatchStatus,
}

impl MeasurementBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementBatchStatus) -> bool {
        match self.status {
            MeasurementBatchStatus::Planned => {
                return next == MeasurementBatchStatus::Completed;
            }
            MeasurementBatchStatus::Scheduled => {
                return next == MeasurementBatchStatus::Completed;
            }
            MeasurementBatchStatus::InProgress => {
                match next {
                    MeasurementBatchStatus::Completed | MeasurementBatchStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementBatchStatus::Completed => {
                match next {
                    MeasurementBatchStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: MeasurementBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement batch", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> MeasurementBatch {
        MeasurementBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            currency: self.currency.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a measurement batch in status planned.
pub fn new_measurement_batch(code: &str, title: &str) -> MeasurementBatch {
    MeasurementBatch {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        currency: "".to_string(),
        tenant_id: "".to_string(),
        status: MeasurementBatchStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_measurement_batch("code-67", "title-459");
        assert_eq!(item.status, MeasurementBatchStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_measurement_batch("code-67", "title-459");
        if item.move_to(MeasurementBatchStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementBatchStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_measurement_batch("code-67", "title-459");
        item.status = MeasurementBatchStatus::Cancelled;
        assert!(
            item.move_to(MeasurementBatchStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
