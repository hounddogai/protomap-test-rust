use super::*;

use acme_kit::text;

/// Where a primary slo reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySloReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimarySloReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimarySloReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary slo reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloReservation {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub created_at: i64,
    pub name: String,
    pub visible: bool,
    pub status: PrimarySloReservationStatus,
}

impl PrimarySloReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySloReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySloReservationStatus) -> bool {
        match self.status {
            PrimarySloReservationStatus::Requested => {
                return next == PrimarySloReservationStatus::Granted;
            }
            PrimarySloReservationStatus::Granted => {
                return next == PrimarySloReservationStatus::Expired;
            }
            PrimarySloReservationStatus::Revoked => {
                return next == PrimarySloReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimarySloReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary slo reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary slo reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> PrimarySloReservation {
        PrimarySloReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            created_at: self.created_at,
            name: self.name.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary slo reservation in status requested.
pub fn new_primary_slo_reservation(code: &str, label: &str) -> PrimarySloReservation {
    PrimarySloReservation {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        created_at: 0,
        name: "".to_string(),
        visible: false,
        status: PrimarySloReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_slo_reservation("code-610", "label-858");
        assert_eq!(item.status, PrimarySloReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_primary_slo_reservation("code-610", "label-858");
        if item.move_to(PrimarySloReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySloReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_slo_reservation("code-610", "label-858");
        item.status = PrimarySloReservationStatus::Expired;
        assert!(
            item.move_to(PrimarySloReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
