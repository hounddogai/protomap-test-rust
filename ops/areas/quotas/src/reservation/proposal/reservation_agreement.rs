use super::*;

use acme_kit::text;

/// Where a reservation agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReservationAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReservationAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation agreement as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAgreement {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub external_ref: String,
    pub title: String,
    pub threshold: f64,
    pub version: i64,
    pub cost: f64,
    pub status: ReservationAgreementStatus,
}

impl ReservationAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationAgreementStatus) -> bool {
        match self.status {
            ReservationAgreementStatus::Created => {
                match next {
                    ReservationAgreementStatus::Published | ReservationAgreementStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationAgreementStatus::Validated => {
                match next {
                    ReservationAgreementStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationAgreementStatus::Published => {
                return next == ReservationAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.version as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationAgreement {
        ReservationAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            external_ref: self.external_ref.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            version: self.version,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a reservation agreement in status created.
pub fn new_reservation_agreement(code: &str, name: &str) -> ReservationAgreement {
    ReservationAgreement {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        external_ref: "".to_string(),
        title: "".to_string(),
        threshold: 0.0,
        version: 0,
        cost: 0.0,
        status: ReservationAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_reservation_agreement("code-383", "name-450");
        assert_eq!(item.status, ReservationAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_reservation_agreement("code-383", "name-450");
        if item.move_to(ReservationAgreementStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationAgreementStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_reservation_agreement("code-383", "name-450");
        item.status = ReservationAgreementStatus::Retired;
        assert!(
            item.move_to(ReservationAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
