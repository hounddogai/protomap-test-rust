use super::*;

use acme_kit::text;

/// Where a seasonal canary reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalCanaryReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalCanaryReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalCanaryReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal canary reservation as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryReservation {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub active: bool,
    pub checksum: String,
    pub amount: f64,
    pub minimum: f64,
    pub revision: i64,
    pub weight: f64,
    pub floor_value: f64,
    pub status: SeasonalCanaryReservationStatus,
}

impl SeasonalCanaryReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalCanaryReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalCanaryReservationStatus) -> bool {
        match self.status {
            SeasonalCanaryReservationStatus::Proposed => {
                return next == SeasonalCanaryReservationStatus::Approved
                    || next == SeasonalCanaryReservationStatus::Withdrawn;
            }
            SeasonalCanaryReservationStatus::Approved => {
                match next {
                    SeasonalCanaryReservationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCanaryReservationStatus::Rejected => {
                return next == SeasonalCanaryReservationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalCanaryReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal canary reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.amount + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalCanaryReservation {
        SeasonalCanaryReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            active: self.active,
            checksum: self.checksum.clone(),
            amount: self.amount,
            minimum: self.minimum,
            revision: self.revision,
            weight: self.weight,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal canary reservation in status proposed.
pub fn new_seasonal_canary_reservation(code: &str, name: &str) -> SeasonalCanaryReservation {
    SeasonalCanaryReservation {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        active: false,
        checksum: "".to_string(),
        amount: 0.0,
        minimum: 0.0,
        revision: 0,
        weight: 0.0,
        floor_value: 0.0,
        status: SeasonalCanaryReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_canary_reservation("code-216", "name-789");
        assert_eq!(item.status, SeasonalCanaryReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_seasonal_canary_reservation("code-216", "name-789");
        if item.move_to(SeasonalCanaryReservationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalCanaryReservationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_canary_reservation("code-216", "name-789");
        item.status = SeasonalCanaryReservationStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalCanaryReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
