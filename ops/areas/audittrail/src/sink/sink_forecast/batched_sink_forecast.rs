use super::*;

use acme_kit::text;

/// Where a batched sink forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSinkForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedSinkForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedSinkForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched sink forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkForecast {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub owner_id: String,
    pub capacity: i64,
    pub discount: f64,
    pub amount: f64,
    pub ceiling: f64,
    pub version: i64,
    pub pinned: bool,
    pub phone: String,
    pub status: BatchedSinkForecastStatus,
}

impl BatchedSinkForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSinkForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSinkForecastStatus) -> bool {
        match self.status {
            BatchedSinkForecastStatus::Proposed => {
                return next == BatchedSinkForecastStatus::Rejected;
            }
            BatchedSinkForecastStatus::Approved => {
                match next {
                    BatchedSinkForecastStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSinkForecastStatus::Rejected => {
                return next == BatchedSinkForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSinkForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched sink forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.amount - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSinkForecast {
        BatchedSinkForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            owner_id: self.owner_id.clone(),
            capacity: self.capacity,
            discount: self.discount,
            amount: self.amount,
            ceiling: self.ceiling,
            version: self.version,
            pinned: self.pinned,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a batched sink forecast in status proposed.
pub fn new_batched_sink_forecast(code: &str, name: &str) -> BatchedSinkForecast {
    BatchedSinkForecast {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        owner_id: "".to_string(),
        capacity: 0,
        discount: 0.0,
        amount: 0.0,
        ceiling: 0.0,
        version: 0,
        pinned: false,
        phone: "".to_string(),
        status: BatchedSinkForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_sink_forecast("code-697", "name-756");
        assert_eq!(item.status, BatchedSinkForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_batched_sink_forecast("code-697", "name-756");
        if item.move_to(BatchedSinkForecastStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSinkForecastStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_sink_forecast("code-697", "name-756");
        item.status = BatchedSinkForecastStatus::Withdrawn;
        assert!(
            item.move_to(BatchedSinkForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
