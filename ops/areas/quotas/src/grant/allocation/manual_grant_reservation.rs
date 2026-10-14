use super::*;

use acme_kit::text;

/// Where a manual grant reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualGrantReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualGrantReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualGrantReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual grant reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantReservation {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub host: String,
    pub channel: String,
    pub ceiling: f64,
    pub revision: i64,
    pub attempts: i64,
    pub fingerprint: String,
    pub status: ManualGrantReservationStatus,
}

impl ManualGrantReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualGrantReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualGrantReservationStatus) -> bool {
        match self.status {
            ManualGrantReservationStatus::Created => {
                match next {
                    ManualGrantReservationStatus::Validated | ManualGrantReservationStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualGrantReservationStatus::Validated => {
                return next == ManualGrantReservationStatus::Published;
            }
            ManualGrantReservationStatus::Published => {
                return next == ManualGrantReservationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualGrantReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual grant reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ceiling - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualGrantReservation {
        ManualGrantReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            host: self.host.clone(),
            channel: self.channel.clone(),
            ceiling: self.ceiling,
            revision: self.revision,
            attempts: self.attempts,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual grant reservation in status created.
pub fn new_manual_grant_reservation(code: &str, label: &str) -> ManualGrantReservation {
    ManualGrantReservation {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        host: "".to_string(),
        channel: "".to_string(),
        ceiling: 0.0,
        revision: 0,
        attempts: 0,
        fingerprint: "".to_string(),
        status: ManualGrantReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_grant_reservation("code-913", "label-622");
        assert_eq!(item.status, ManualGrantReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_manual_grant_reservation("code-913", "label-622");
        if item.move_to(ManualGrantReservationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualGrantReservationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_grant_reservation("code-913", "label-622");
        item.status = ManualGrantReservationStatus::Retired;
        assert!(
            item.move_to(ManualGrantReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
