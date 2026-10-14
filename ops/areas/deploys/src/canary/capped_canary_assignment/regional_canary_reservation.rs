use super::*;

use acme_kit::text;

/// Where a regional canary reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalCanaryReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalCanaryReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RegionalCanaryReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional canary reservation as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryReservation {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub ttl_seconds: i64,
    pub fingerprint: String,
    pub pinned: bool,
    pub host: String,
    pub status: RegionalCanaryReservationStatus,
}

impl RegionalCanaryReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalCanaryReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalCanaryReservationStatus) -> bool {
        match self.status {
            RegionalCanaryReservationStatus::Planned => {
                match next {
                    RegionalCanaryReservationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalCanaryReservationStatus::Scheduled => {
                return next == RegionalCanaryReservationStatus::Cancelled;
            }
            RegionalCanaryReservationStatus::InProgress => {
                return next == RegionalCanaryReservationStatus::Completed
                    || next == RegionalCanaryReservationStatus::Cancelled;
            }
            RegionalCanaryReservationStatus::Completed => {
                match next {
                    RegionalCanaryReservationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionalCanaryReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional canary reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional canary reservation", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.burst as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalCanaryReservation {
        RegionalCanaryReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            ttl_seconds: self.ttl_seconds,
            fingerprint: self.fingerprint.clone(),
            pinned: self.pinned,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional canary reservation in status planned.
pub fn new_regional_canary_reservation(code: &str, name: &str) -> RegionalCanaryReservation {
    RegionalCanaryReservation {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        ttl_seconds: 0,
        fingerprint: "".to_string(),
        pinned: false,
        host: "".to_string(),
        status: RegionalCanaryReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_regional_canary_reservation("code-319", "name-211");
        assert_eq!(item.status, RegionalCanaryReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_regional_canary_reservation("code-319", "name-211");
        if item.move_to(RegionalCanaryReservationStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalCanaryReservationStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_regional_canary_reservation("code-319", "name-211");
        item.status = RegionalCanaryReservationStatus::Cancelled;
        assert!(
            item.move_to(RegionalCanaryReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
