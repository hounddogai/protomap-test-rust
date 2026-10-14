use super::*;

use acme_kit::text;

/// Where a rolling probe reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingProbeReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingProbeReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingProbeReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling probe reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeReservation {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub country: String,
    pub cluster: String,
    pub status: RollingProbeReservationStatus,
}

impl RollingProbeReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingProbeReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingProbeReservationStatus) -> bool {
        match self.status {
            RollingProbeReservationStatus::Requested => {
                match next {
                    RollingProbeReservationStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingProbeReservationStatus::Granted => {
                match next {
                    RollingProbeReservationStatus::Revoked | RollingProbeReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingProbeReservationStatus::Revoked => {
                return next == RollingProbeReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingProbeReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling probe reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling probe reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> RollingProbeReservation {
        RollingProbeReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            country: self.country.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling probe reservation in status requested.
pub fn new_rolling_probe_reservation(code: &str, name: &str) -> RollingProbeReservation {
    RollingProbeReservation {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        country: "".to_string(),
        cluster: "".to_string(),
        status: RollingProbeReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_probe_reservation("code-712", "name-618");
        assert_eq!(item.status, RollingProbeReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rolling_probe_reservation("code-712", "name-618");
        if item.move_to(RollingProbeReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingProbeReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_probe_reservation("code-712", "name-618");
        item.status = RollingProbeReservationStatus::Expired;
        assert!(
            item.move_to(RollingProbeReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
