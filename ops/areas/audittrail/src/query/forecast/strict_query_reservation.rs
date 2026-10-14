use super::*;

use acme_kit::text;

/// Where a strict query reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictQueryReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictQueryReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictQueryReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict query reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryReservation {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub threshold: f64,
    pub timezone: String,
    pub quantity: i64,
    pub revision: i64,
    pub discount: f64,
    pub status: StrictQueryReservationStatus,
}

impl StrictQueryReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictQueryReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictQueryReservationStatus) -> bool {
        match self.status {
            StrictQueryReservationStatus::Requested => {
                match next {
                    StrictQueryReservationStatus::Granted | StrictQueryReservationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictQueryReservationStatus::Granted => {
                return next == StrictQueryReservationStatus::Revoked;
            }
            StrictQueryReservationStatus::Revoked => {
                return next == StrictQueryReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictQueryReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict query reservation {} cannot move from {} to {}",
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

    pub fn effective_load(&self) -> f64 {
        (self.discount * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictQueryReservation {
        StrictQueryReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            threshold: self.threshold,
            timezone: self.timezone.clone(),
            quantity: self.quantity,
            revision: self.revision,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict query reservation in status requested.
pub fn new_strict_query_reservation(code: &str, label: &str) -> StrictQueryReservation {
    StrictQueryReservation {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        threshold: 0.0,
        timezone: "".to_string(),
        quantity: 0,
        revision: 0,
        discount: 0.0,
        status: StrictQueryReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_query_reservation("code-390", "label-414");
        assert_eq!(item.status, StrictQueryReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_strict_query_reservation("code-390", "label-414");
        if item.move_to(StrictQueryReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictQueryReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_query_reservation("code-390", "label-414");
        item.status = StrictQueryReservationStatus::Expired;
        assert!(
            item.move_to(StrictQueryReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
