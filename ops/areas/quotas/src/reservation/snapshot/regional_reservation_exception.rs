use super::*;

use acme_kit::text;

/// Where a regional reservation exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalReservationExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalReservationExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RegionalReservationExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional reservation exception as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationException {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub quota_units: i64,
    pub status: RegionalReservationExceptionStatus,
}

impl RegionalReservationException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalReservationExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalReservationExceptionStatus) -> bool {
        match self.status {
            RegionalReservationExceptionStatus::Idle => {
                return next == RegionalReservationExceptionStatus::Warming;
            }
            RegionalReservationExceptionStatus::Warming => {
                match next {
                    RegionalReservationExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalReservationExceptionStatus::Ready => {
                return next == RegionalReservationExceptionStatus::Stopped;
            }
            RegionalReservationExceptionStatus::Draining => {
                return next == RegionalReservationExceptionStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalReservationExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional reservation exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quota_units as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalReservationException {
        RegionalReservationException {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional reservation exception in status idle.
pub fn new_regional_reservation_exception(code: &str, label: &str) -> RegionalReservationException {
    RegionalReservationException {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        title: "".to_string(),
        ttl_seconds: 0,
        quota_units: 0,
        status: RegionalReservationExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_reservation_exception("code-346", "label-899");
        assert_eq!(item.status, RegionalReservationExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_regional_reservation_exception("code-346", "label-899");
        if item.move_to(RegionalReservationExceptionStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalReservationExceptionStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_reservation_exception("code-346", "label-899");
        item.status = RegionalReservationExceptionStatus::Stopped;
        assert!(
            item.move_to(RegionalReservationExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
