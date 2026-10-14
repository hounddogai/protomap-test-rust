use super::*;

use acme_kit::text;

/// Where a reservation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ReservationTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ReservationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTarget {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub priority: i64,
    pub quantity: i64,
    pub issuer: String,
    pub duration_minutes: i64,
    pub cluster: String,
    pub key_id: String,
    pub status: ReservationTargetStatus,
}

impl ReservationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationTargetStatus) -> bool {
        match self.status {
            ReservationTargetStatus::Requested => {
                match next {
                    ReservationTargetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationTargetStatus::Granted => {
                return next == ReservationTargetStatus::Expired;
            }
            ReservationTargetStatus::Revoked => {
                return next == ReservationTargetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "reservation target", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationTarget {
        ReservationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            priority: self.priority,
            quantity: self.quantity,
            issuer: self.issuer.clone(),
            duration_minutes: self.duration_minutes,
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a reservation target in status requested.
pub fn new_reservation_target(code: &str, name: &str) -> ReservationTarget {
    ReservationTarget {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        priority: 0,
        quantity: 0,
        issuer: "".to_string(),
        duration_minutes: 0,
        cluster: "".to_string(),
        key_id: "".to_string(),
        status: ReservationTargetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_reservation_target("code-505", "name-413");
        assert_eq!(item.status, ReservationTargetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_reservation_target("code-505", "name-413");
        if item.move_to(ReservationTargetStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationTargetStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_reservation_target("code-505", "name-413");
        item.status = ReservationTargetStatus::Expired;
        assert!(
            item.move_to(ReservationTargetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
