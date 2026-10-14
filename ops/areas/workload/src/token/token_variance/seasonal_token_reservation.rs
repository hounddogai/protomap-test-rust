use super::*;

use acme_kit::text;

/// Where a seasonal token reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTokenReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalTokenReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalTokenReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal token reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenReservation {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub quota_units: i64,
    pub country: String,
    pub status: SeasonalTokenReservationStatus,
}

impl SeasonalTokenReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTokenReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTokenReservationStatus) -> bool {
        match self.status {
            SeasonalTokenReservationStatus::Proposed => {
                return next == SeasonalTokenReservationStatus::Approved
                    || next == SeasonalTokenReservationStatus::Rejected;
            }
            SeasonalTokenReservationStatus::Approved => {
                match next {
                    SeasonalTokenReservationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenReservationStatus::Rejected => {
                match next {
                    SeasonalTokenReservationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SeasonalTokenReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal token reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> SeasonalTokenReservation {
        SeasonalTokenReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            quota_units: self.quota_units,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal token reservation in status proposed.
pub fn new_seasonal_token_reservation(code: &str, title: &str) -> SeasonalTokenReservation {
    SeasonalTokenReservation {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        quota_units: 0,
        country: "".to_string(),
        status: SeasonalTokenReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_token_reservation("code-863", "title-757");
        assert_eq!(item.status, SeasonalTokenReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_seasonal_token_reservation("code-863", "title-757");
        if item.move_to(SeasonalTokenReservationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalTokenReservationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_token_reservation("code-863", "title-757");
        item.status = SeasonalTokenReservationStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalTokenReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
