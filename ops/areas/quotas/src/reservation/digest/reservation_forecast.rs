use super::*;

use acme_kit::text;

/// Where a reservation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ReservationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ReservationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One reservation forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationForecast {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub contact_email: String,
    pub quota_units: i64,
    pub retries: i64,
    pub status: ReservationForecastStatus,
}

impl ReservationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationForecastStatus) -> bool {
        match self.status {
            ReservationForecastStatus::Requested => {
                match next {
                    ReservationForecastStatus::Granted | ReservationForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationForecastStatus::Granted => {
                return next == ReservationForecastStatus::Revoked || next == ReservationForecastStatus::Expired;
            }
            ReservationForecastStatus::Revoked => {
                match next {
                    ReservationForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: ReservationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "reservation forecast", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.retries as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationForecast {
        ReservationForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            contact_email: self.contact_email.clone(),
            quota_units: self.quota_units,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a reservation forecast in status requested.
pub fn new_reservation_forecast(code: &str, label: &str) -> ReservationForecast {
    ReservationForecast {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        contact_email: "".to_string(),
        quota_units: 0,
        retries: 0,
        status: ReservationForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_reservation_forecast("code-951", "label-932");
        assert_eq!(item.status, ReservationForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_reservation_forecast("code-951", "label-932");
        if item.move_to(ReservationForecastStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationForecastStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_reservation_forecast("code-951", "label-932");
        item.status = ReservationForecastStatus::Expired;
        assert!(
            item.move_to(ReservationForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
