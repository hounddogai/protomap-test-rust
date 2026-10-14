use super::*;

use acme_kit::text;

/// Where a reservation window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ReservationWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ReservationWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation window as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationWindow {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub country: String,
    pub notes: String,
    pub amount: f64,
    pub status: ReservationWindowStatus,
}

impl ReservationWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationWindowStatus) -> bool {
        match self.status {
            ReservationWindowStatus::Requested => {
                return next == ReservationWindowStatus::Granted || next == ReservationWindowStatus::Expired;
            }
            ReservationWindowStatus::Granted => {
                return next == ReservationWindowStatus::Revoked || next == ReservationWindowStatus::Expired;
            }
            ReservationWindowStatus::Revoked => {
                return next == ReservationWindowStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> ReservationWindow {
        ReservationWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            country: self.country.clone(),
            notes: self.notes.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a reservation window in status requested.
pub fn new_reservation_window(code: &str, label: &str) -> ReservationWindow {
    ReservationWindow {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        country: "".to_string(),
        notes: "".to_string(),
        amount: 0.0,
        status: ReservationWindowStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_reservation_window("code-456", "label-740");
        assert_eq!(item.status, ReservationWindowStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_reservation_window("code-456", "label-740");
        if item.move_to(ReservationWindowStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationWindowStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_reservation_window("code-456", "label-740");
        item.status = ReservationWindowStatus::Expired;
        assert!(
            item.move_to(ReservationWindowStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
