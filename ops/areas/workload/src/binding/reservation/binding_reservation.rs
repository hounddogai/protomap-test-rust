use super::*;

use acme_kit::text;

/// Where a binding reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BindingReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BindingReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingReservation {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub pinned: bool,
    pub channel: String,
    pub score: f64,
    pub status: BindingReservationStatus,
}

impl BindingReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingReservationStatus) -> bool {
        match self.status {
            BindingReservationStatus::Requested => {
                return next == BindingReservationStatus::Granted || next == BindingReservationStatus::Revoked;
            }
            BindingReservationStatus::Granted => {
                match next {
                    BindingReservationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingReservationStatus::Revoked => {
                return next == BindingReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> BindingReservation {
        BindingReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            pinned: self.pinned,
            channel: self.channel.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a binding reservation in status requested.
pub fn new_binding_reservation(code: &str, label: &str) -> BindingReservation {
    BindingReservation {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        pinned: false,
        channel: "".to_string(),
        score: 0.0,
        status: BindingReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_binding_reservation("code-682", "label-247");
        assert_eq!(item.status, BindingReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_binding_reservation("code-682", "label-247");
        if item.move_to(BindingReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_binding_reservation("code-682", "label-247");
        item.status = BindingReservationStatus::Expired;
        assert!(
            item.move_to(BindingReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
