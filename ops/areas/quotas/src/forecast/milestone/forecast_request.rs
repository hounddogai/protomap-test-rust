use super::*;

use acme_kit::text;

/// Where a forecast request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastRequestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ForecastRequestStatus {
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

impl std::fmt::Display for ForecastRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRequest {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub discount: f64,
    pub active: bool,
    pub capacity: i64,
    pub rotation_days: i64,
    pub rate: f64,
    pub minimum: f64,
    pub status: ForecastRequestStatus,
}

impl ForecastRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastRequestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastRequestStatus) -> bool {
        match self.status {
            ForecastRequestStatus::Planned => {
                match next {
                    ForecastRequestStatus::InProgress | ForecastRequestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastRequestStatus::Scheduled => {
                return next == ForecastRequestStatus::Completed;
            }
            ForecastRequestStatus::InProgress => {
                match next {
                    ForecastRequestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastRequestStatus::Completed => {
                match next {
                    ForecastRequestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ForecastRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rotation_days as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastRequest {
        ForecastRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            discount: self.discount,
            active: self.active,
            capacity: self.capacity,
            rotation_days: self.rotation_days,
            rate: self.rate,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a forecast request in status planned.
pub fn new_forecast_request(code: &str, name: &str) -> ForecastRequest {
    ForecastRequest {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        discount: 0.0,
        active: false,
        capacity: 0,
        rotation_days: 0,
        rate: 0.0,
        minimum: 0.0,
        status: ForecastRequestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_forecast_request("code-417", "name-659");
        assert_eq!(item.status, ForecastRequestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_forecast_request("code-417", "name-659");
        if item.move_to(ForecastRequestStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastRequestStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_forecast_request("code-417", "name-659");
        item.status = ForecastRequestStatus::Cancelled;
        assert!(
            item.move_to(ForecastRequestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
