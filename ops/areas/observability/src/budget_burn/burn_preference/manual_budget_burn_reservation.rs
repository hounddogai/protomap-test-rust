use super::*;

use acme_kit::text;

/// Where a manual budget burn reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualBudgetBurnReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualBudgetBurnReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualBudgetBurnReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual budget burn reservation as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnReservation {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub contact_email: String,
    pub minimum: f64,
    pub country: String,
    pub status: ManualBudgetBurnReservationStatus,
}

impl ManualBudgetBurnReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualBudgetBurnReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualBudgetBurnReservationStatus) -> bool {
        match self.status {
            ManualBudgetBurnReservationStatus::Requested => {
                return next == ManualBudgetBurnReservationStatus::Granted
                    || next == ManualBudgetBurnReservationStatus::Expired;
            }
            ManualBudgetBurnReservationStatus::Granted => {
                return next == ManualBudgetBurnReservationStatus::Expired;
            }
            ManualBudgetBurnReservationStatus::Revoked => {
                return next == ManualBudgetBurnReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualBudgetBurnReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual budget burn reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> ManualBudgetBurnReservation {
        ManualBudgetBurnReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            contact_email: self.contact_email.clone(),
            minimum: self.minimum,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a manual budget burn reservation in status requested.
pub fn new_manual_budget_burn_reservation(code: &str, name: &str) -> ManualBudgetBurnReservation {
    ManualBudgetBurnReservation {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        contact_email: "".to_string(),
        minimum: 0.0,
        country: "".to_string(),
        status: ManualBudgetBurnReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_budget_burn_reservation("code-512", "name-376");
        assert_eq!(item.status, ManualBudgetBurnReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_budget_burn_reservation("code-512", "name-376");
        if item.move_to(ManualBudgetBurnReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualBudgetBurnReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_budget_burn_reservation("code-512", "name-376");
        item.status = ManualBudgetBurnReservationStatus::Expired;
        assert!(
            item.move_to(ManualBudgetBurnReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
