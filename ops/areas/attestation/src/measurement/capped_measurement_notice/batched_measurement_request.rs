use super::*;

use acme_kit::text;

/// Where a batched measurement request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedMeasurementRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedMeasurementRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedMeasurementRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched measurement request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementRequest {
    pub code: String,
    pub name: String,
    pub title: String,
    pub owner_id: String,
    pub external_ref: String,
    pub grace_days: i64,
    pub status: BatchedMeasurementRequestStatus,
}

impl BatchedMeasurementRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedMeasurementRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedMeasurementRequestStatus) -> bool {
        match self.status {
            BatchedMeasurementRequestStatus::Draft => {
                match next {
                    BatchedMeasurementRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedMeasurementRequestStatus::Pending => {
                match next {
                    BatchedMeasurementRequestStatus::Active | BatchedMeasurementRequestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedMeasurementRequestStatus::Active => {
                return next == BatchedMeasurementRequestStatus::Suspended;
            }
            BatchedMeasurementRequestStatus::Suspended => {
                match next {
                    BatchedMeasurementRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedMeasurementRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched measurement request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> BatchedMeasurementRequest {
        BatchedMeasurementRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            external_ref: self.external_ref.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched measurement request in status draft.
pub fn new_batched_measurement_request(code: &str, name: &str) -> BatchedMeasurementRequest {
    BatchedMeasurementRequest {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        owner_id: "".to_string(),
        external_ref: "".to_string(),
        grace_days: 0,
        status: BatchedMeasurementRequestStatus::Draft,
    }
}
