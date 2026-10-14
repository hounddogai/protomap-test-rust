use super::*;

use acme_kit::text;

/// Where a batched forecast reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedForecastReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedForecastReservationStatus {
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

impl std::fmt::Display for BatchedForecastReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched forecast reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReservation {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub category: String,
    pub host: String,
    pub quota_units: i64,
    pub floor_value: f64,
    pub currency: String,
    pub summary: String,
    pub priority: i64,
    pub timezone: String,
    pub status: BatchedForecastReservationStatus,
}

impl BatchedForecastReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedForecastReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedForecastReservationStatus) -> bool {
        match self.status {
            BatchedForecastReservationStatus::Draft => {
                match next {
                    BatchedForecastReservationStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedForecastReservationStatus::Pending => {
                return next == BatchedForecastReservationStatus::Active;
            }
            BatchedForecastReservationStatus::Active => {
                match next {
                    BatchedForecastReservationStatus::Suspended | BatchedForecastReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedForecastReservationStatus::Suspended => {
                return next == BatchedForecastReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedForecastReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched forecast reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.retries as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedForecastReservation {
        BatchedForecastReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            category: self.category.clone(),
            host: self.host.clone(),
            quota_units: self.quota_units,
            floor_value: self.floor_value,
            currency: self.currency.clone(),
            summary: self.summary.clone(),
            priority: self.priority,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched forecast reservation in status draft.
pub fn new_batched_forecast_reservation(code: &str, label: &str) -> BatchedForecastReservation {
    BatchedForecastReservation {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        category: "".to_string(),
        host: "".to_string(),
        quota_units: 0,
        floor_value: 0.0,
        currency: "".to_string(),
        summary: "".to_string(),
        priority: 0,
        timezone: "".to_string(),
        status: BatchedForecastReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_forecast_reservation("code-774", "label-786");
        assert_eq!(item.status, BatchedForecastReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_batched_forecast_reservation("code-774", "label-786");
        if item.move_to(BatchedForecastReservationStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedForecastReservationStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_forecast_reservation("code-774", "label-786");
        item.status = BatchedForecastReservationStatus::Closed;
        assert!(
            item.move_to(BatchedForecastReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
