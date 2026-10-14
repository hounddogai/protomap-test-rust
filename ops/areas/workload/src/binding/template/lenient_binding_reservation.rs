use super::*;

use acme_kit::text;

/// Where a lenient binding reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientBindingReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientBindingReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientBindingReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient binding reservation as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingReservation {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub contact_email: String,
    pub quota_units: i64,
    pub sequence: i64,
    pub phone: String,
    pub grace_days: i64,
    pub rate: f64,
    pub burst: i64,
    pub cost: f64,
    pub status: LenientBindingReservationStatus,
}

impl LenientBindingReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientBindingReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientBindingReservationStatus) -> bool {
        match self.status {
            LenientBindingReservationStatus::Requested => {
                return next == LenientBindingReservationStatus::Expired;
            }
            LenientBindingReservationStatus::Granted => {
                match next {
                    LenientBindingReservationStatus::Revoked | LenientBindingReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientBindingReservationStatus::Revoked => {
                match next {
                    LenientBindingReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientBindingReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient binding reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.sequence as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientBindingReservation {
        LenientBindingReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            contact_email: self.contact_email.clone(),
            quota_units: self.quota_units,
            sequence: self.sequence,
            phone: self.phone.clone(),
            grace_days: self.grace_days,
            rate: self.rate,
            burst: self.burst,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient binding reservation in status requested.
pub fn new_lenient_binding_reservation(code: &str, name: &str) -> LenientBindingReservation {
    LenientBindingReservation {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        contact_email: "".to_string(),
        quota_units: 0,
        sequence: 0,
        phone: "".to_string(),
        grace_days: 0,
        rate: 0.0,
        burst: 0,
        cost: 0.0,
        status: LenientBindingReservationStatus::Requested,
    }
}
