use super::*;

use acme_kit::text;

/// Where a reservation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationProfileStatus {
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

impl std::fmt::Display for ReservationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One reservation profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationProfile {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub external_ref: String,
    pub retries: i64,
    pub status: ReservationProfileStatus,
}

impl ReservationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationProfileStatus) -> bool {
        match self.status {
            ReservationProfileStatus::Idle => {
                match next {
                    ReservationProfileStatus::Warming | ReservationProfileStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationProfileStatus::Warming => {
                match next {
                    ReservationProfileStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationProfileStatus::Ready => {
                return next == ReservationProfileStatus::Stopped;
            }
            ReservationProfileStatus::Draining => {
                return next == ReservationProfileStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> ReservationProfile {
        ReservationProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            external_ref: self.external_ref.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a reservation profile in status idle.
pub fn new_reservation_profile(code: &str, title: &str) -> ReservationProfile {
    ReservationProfile {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        external_ref: "".to_string(),
        retries: 0,
        status: ReservationProfileStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_reservation_profile("code-676", "title-837");
        assert_eq!(item.status, ReservationProfileStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_reservation_profile("code-676", "title-837");
        if item.move_to(ReservationProfileStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationProfileStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_reservation_profile("code-676", "title-837");
        item.status = ReservationProfileStatus::Stopped;
        assert!(
            item.move_to(ReservationProfileStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
