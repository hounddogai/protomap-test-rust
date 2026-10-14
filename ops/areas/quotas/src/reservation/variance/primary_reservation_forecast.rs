use super::*;

use acme_kit::text;

/// Where a primary reservation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryReservationForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryReservationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryReservationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary reservation forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationForecast {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub contact_email: String,
    pub amount: f64,
    pub scope: String,
    pub cost: f64,
    pub ceiling: f64,
    pub namespace: String,
    pub window_days: i64,
    pub maximum: f64,
    pub status: PrimaryReservationForecastStatus,
}

impl PrimaryReservationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryReservationForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryReservationForecastStatus) -> bool {
        match self.status {
            PrimaryReservationForecastStatus::Proposed => {
                return next == PrimaryReservationForecastStatus::Withdrawn;
            }
            PrimaryReservationForecastStatus::Approved => {
                match next {
                    PrimaryReservationForecastStatus::Rejected | PrimaryReservationForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReservationForecastStatus::Rejected => {
                match next {
                    PrimaryReservationForecastStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimaryReservationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary reservation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary reservation forecast", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.cost - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryReservationForecast {
        PrimaryReservationForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            contact_email: self.contact_email.clone(),
            amount: self.amount,
            scope: self.scope.clone(),
            cost: self.cost,
            ceiling: self.ceiling,
            namespace: self.namespace.clone(),
            window_days: self.window_days,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a primary reservation forecast in status proposed.
pub fn new_primary_reservation_forecast(code: &str, title: &str) -> PrimaryReservationForecast {
    PrimaryReservationForecast {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        contact_email: "".to_string(),
        amount: 0.0,
        scope: "".to_string(),
        cost: 0.0,
        ceiling: 0.0,
        namespace: "".to_string(),
        window_days: 0,
        maximum: 0.0,
        status: PrimaryReservationForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_reservation_forecast("code-968", "title-6");
        assert_eq!(item.status, PrimaryReservationForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_primary_reservation_forecast("code-968", "title-6");
        if item.move_to(PrimaryReservationForecastStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryReservationForecastStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_reservation_forecast("code-968", "title-6");
        item.status = PrimaryReservationForecastStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryReservationForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
