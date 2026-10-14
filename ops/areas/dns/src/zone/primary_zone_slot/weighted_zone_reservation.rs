use super::*;

use acme_kit::text;

/// Where a weighted zone reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedZoneReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedZoneReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedZoneReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted zone reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneReservation {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub expires_at: i64,
    pub attempts: i64,
    pub status: WeightedZoneReservationStatus,
}

impl WeightedZoneReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedZoneReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedZoneReservationStatus) -> bool {
        match self.status {
            WeightedZoneReservationStatus::Requested => {
                return next == WeightedZoneReservationStatus::Granted;
            }
            WeightedZoneReservationStatus::Granted => {
                match next {
                    WeightedZoneReservationStatus::Revoked | WeightedZoneReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedZoneReservationStatus::Revoked => {
                return next == WeightedZoneReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedZoneReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted zone reservation {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.attempts as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedZoneReservation {
        WeightedZoneReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            expires_at: self.expires_at,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted zone reservation in status requested.
pub fn new_weighted_zone_reservation(code: &str, title: &str) -> WeightedZoneReservation {
    WeightedZoneReservation {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        expires_at: 0,
        attempts: 0,
        status: WeightedZoneReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_weighted_zone_reservation("code-622", "title-401");
        assert_eq!(item.status, WeightedZoneReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_weighted_zone_reservation("code-622", "title-401");
        if item.move_to(WeightedZoneReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedZoneReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_weighted_zone_reservation("code-622", "title-401");
        item.status = WeightedZoneReservationStatus::Expired;
        assert!(
            item.move_to(WeightedZoneReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
