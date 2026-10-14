use super::*;

use acme_kit::text;

/// Where a forecast reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ForecastReservationStatus {
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

impl std::fmt::Display for ForecastReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReservation {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub tenant_id: String,
    pub key_id: String,
    pub segment: String,
    pub amount: f64,
    pub namespace: String,
    pub scope: String,
    pub status: ForecastReservationStatus,
}

impl ForecastReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastReservationStatus) -> bool {
        match self.status {
            ForecastReservationStatus::Planned => {
                return next == ForecastReservationStatus::InProgress || next == ForecastReservationStatus::Completed;
            }
            ForecastReservationStatus::Scheduled => {
                return next == ForecastReservationStatus::Completed;
            }
            ForecastReservationStatus::InProgress => {
                match next {
                    ForecastReservationStatus::Completed | ForecastReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastReservationStatus::Completed => {
                return next == ForecastReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "forecast reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> ForecastReservation {
        ForecastReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            tenant_id: self.tenant_id.clone(),
            key_id: self.key_id.clone(),
            segment: self.segment.clone(),
            amount: self.amount,
            namespace: self.namespace.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a forecast reservation in status planned.
pub fn new_forecast_reservation(code: &str, label: &str) -> ForecastReservation {
    ForecastReservation {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        tenant_id: "".to_string(),
        key_id: "".to_string(),
        segment: "".to_string(),
        amount: 0.0,
        namespace: "".to_string(),
        scope: "".to_string(),
        status: ForecastReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_forecast_reservation("code-621", "label-832");
        assert_eq!(item.status, ForecastReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_forecast_reservation("code-621", "label-832");
        if item.move_to(ForecastReservationStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastReservationStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_forecast_reservation("code-621", "label-832");
        item.status = ForecastReservationStatus::Cancelled;
        assert!(
            item.move_to(ForecastReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
