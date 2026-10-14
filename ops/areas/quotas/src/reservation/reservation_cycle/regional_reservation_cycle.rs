use super::*;

use acme_kit::text;

/// Where a regional reservation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalReservationCycleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalReservationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalReservationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional reservation cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationCycle {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub discount: f64,
    pub description: String,
    pub reviewer: String,
    pub country: String,
    pub expires_at: i64,
    pub contact_email: String,
    pub summary: String,
    pub maximum: f64,
    pub status: RegionalReservationCycleStatus,
}

impl RegionalReservationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalReservationCycleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalReservationCycleStatus) -> bool {
        match self.status {
            RegionalReservationCycleStatus::Requested => {
                match next {
                    RegionalReservationCycleStatus::Granted | RegionalReservationCycleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalReservationCycleStatus::Granted => {
                return next == RegionalReservationCycleStatus::Revoked
                    || next == RegionalReservationCycleStatus::Expired;
            }
            RegionalReservationCycleStatus::Revoked => {
                match next {
                    RegionalReservationCycleStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalReservationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional reservation cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.expires_at as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalReservationCycle {
        RegionalReservationCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            discount: self.discount,
            description: self.description.clone(),
            reviewer: self.reviewer.clone(),
            country: self.country.clone(),
            expires_at: self.expires_at,
            contact_email: self.contact_email.clone(),
            summary: self.summary.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional reservation cycle in status requested.
pub fn new_regional_reservation_cycle(code: &str, label: &str) -> RegionalReservationCycle {
    RegionalReservationCycle {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        discount: 0.0,
        description: "".to_string(),
        reviewer: "".to_string(),
        country: "".to_string(),
        expires_at: 0,
        contact_email: "".to_string(),
        summary: "".to_string(),
        maximum: 0.0,
        status: RegionalReservationCycleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_reservation_cycle("code-406", "label-916");
        assert_eq!(item.status, RegionalReservationCycleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_regional_reservation_cycle("code-406", "label-916");
        if item.move_to(RegionalReservationCycleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalReservationCycleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_reservation_cycle("code-406", "label-916");
        item.status = RegionalReservationCycleStatus::Expired;
        assert!(
            item.move_to(RegionalReservationCycleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
