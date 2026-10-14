use super::*;

use acme_kit::text;

/// Where a weighted alert reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedAlertReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedAlertReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedAlertReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted alert reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertReservation {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub ceiling: f64,
    pub amount: f64,
    pub status: WeightedAlertReservationStatus,
}

impl WeightedAlertReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedAlertReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedAlertReservationStatus) -> bool {
        match self.status {
            WeightedAlertReservationStatus::Created => {
                return next == WeightedAlertReservationStatus::Published
                    || next == WeightedAlertReservationStatus::Retired;
            }
            WeightedAlertReservationStatus::Validated => {
                return next == WeightedAlertReservationStatus::Published;
            }
            WeightedAlertReservationStatus::Published => {
                return next == WeightedAlertReservationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedAlertReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted alert reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted alert reservation", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.amount + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedAlertReservation {
        WeightedAlertReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted alert reservation in status created.
pub fn new_weighted_alert_reservation(code: &str, name: &str) -> WeightedAlertReservation {
    WeightedAlertReservation {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        ceiling: 0.0,
        amount: 0.0,
        status: WeightedAlertReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_alert_reservation("code-361", "name-824");
        assert_eq!(item.status, WeightedAlertReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_weighted_alert_reservation("code-361", "name-824");
        if item.move_to(WeightedAlertReservationStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedAlertReservationStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_alert_reservation("code-361", "name-824");
        item.status = WeightedAlertReservationStatus::Retired;
        assert!(
            item.move_to(WeightedAlertReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
