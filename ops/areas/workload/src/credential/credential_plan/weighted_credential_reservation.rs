use super::*;

use acme_kit::text;

/// Where a weighted credential reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedCredentialReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedCredentialReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedCredentialReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted credential reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialReservation {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub active: bool,
    pub region: String,
    pub country: String,
    pub quantity: i64,
    pub status: WeightedCredentialReservationStatus,
}

impl WeightedCredentialReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedCredentialReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedCredentialReservationStatus) -> bool {
        match self.status {
            WeightedCredentialReservationStatus::Proposed => {
                match next {
                    WeightedCredentialReservationStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedCredentialReservationStatus::Approved => {
                return next == WeightedCredentialReservationStatus::Withdrawn;
            }
            WeightedCredentialReservationStatus::Rejected => {
                return next == WeightedCredentialReservationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedCredentialReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted credential reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> WeightedCredentialReservation {
        WeightedCredentialReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            active: self.active,
            region: self.region.clone(),
            country: self.country.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a weighted credential reservation in status proposed.
pub fn new_weighted_credential_reservation(code: &str, title: &str) -> WeightedCredentialReservation {
    WeightedCredentialReservation {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        active: false,
        region: "".to_string(),
        country: "".to_string(),
        quantity: 0,
        status: WeightedCredentialReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_credential_reservation("code-644", "title-516");
        assert_eq!(item.status, WeightedCredentialReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_weighted_credential_reservation("code-644", "title-516");
        if item.move_to(WeightedCredentialReservationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedCredentialReservationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_credential_reservation("code-644", "title-516");
        item.status = WeightedCredentialReservationStatus::Withdrawn;
        assert!(
            item.move_to(WeightedCredentialReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
