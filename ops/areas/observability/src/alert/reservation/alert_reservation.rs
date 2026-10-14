use super::*;

use acme_kit::text;

/// Where a alert reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AlertReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AlertReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert reservation as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertReservation {
    pub code: String,
    pub name: String,
    pub region: String,
    pub ratio: f64,
    pub locale: String,
    pub floor_value: f64,
    pub approved: bool,
    pub discount: f64,
    pub status: AlertReservationStatus,
}

impl AlertReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertReservationStatus) -> bool {
        match self.status {
            AlertReservationStatus::New => {
                match next {
                    AlertReservationStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertReservationStatus::Queued => {
                match next {
                    AlertReservationStatus::Running | AlertReservationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertReservationStatus::Running => {
                match next {
                    AlertReservationStatus::Done | AlertReservationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertReservationStatus::Done => {
                match next {
                    AlertReservationStatus::Failed => {
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

    pub fn move_to(&mut self, next: AlertReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "alert reservation", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ratio + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertReservation {
        AlertReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            ratio: self.ratio,
            locale: self.locale.clone(),
            floor_value: self.floor_value,
            approved: self.approved,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a alert reservation in status new.
pub fn new_alert_reservation(code: &str, name: &str) -> AlertReservation {
    AlertReservation {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        ratio: 0.0,
        locale: "".to_string(),
        floor_value: 0.0,
        approved: false,
        discount: 0.0,
        status: AlertReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_alert_reservation("code-914", "name-139");
        assert_eq!(item.status, AlertReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_alert_reservation("code-914", "name-139");
        if item.move_to(AlertReservationStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertReservationStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_alert_reservation("code-914", "name-139");
        item.status = AlertReservationStatus::Failed;
        assert!(
            item.move_to(AlertReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
