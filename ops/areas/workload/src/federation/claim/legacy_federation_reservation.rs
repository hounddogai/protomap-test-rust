use super::*;

use acme_kit::text;

/// Where a legacy federation reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFederationReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyFederationReservationStatus {
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

impl std::fmt::Display for LegacyFederationReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy federation reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationReservation {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub ends_at: i64,
    pub minimum: f64,
    pub owner_id: String,
    pub host: String,
    pub status: LegacyFederationReservationStatus,
}

impl LegacyFederationReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFederationReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFederationReservationStatus) -> bool {
        match self.status {
            LegacyFederationReservationStatus::Idle => {
                return next == LegacyFederationReservationStatus::Warming;
            }
            LegacyFederationReservationStatus::Warming => {
                return next == LegacyFederationReservationStatus::Ready
                    || next == LegacyFederationReservationStatus::Draining;
            }
            LegacyFederationReservationStatus::Ready => {
                match next {
                    LegacyFederationReservationStatus::Draining | LegacyFederationReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFederationReservationStatus::Draining => {
                return next == LegacyFederationReservationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFederationReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy federation reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy federation reservation", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ends_at as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFederationReservation {
        LegacyFederationReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            ends_at: self.ends_at,
            minimum: self.minimum,
            owner_id: self.owner_id.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy federation reservation in status idle.
pub fn new_legacy_federation_reservation(code: &str, name: &str) -> LegacyFederationReservation {
    LegacyFederationReservation {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        ends_at: 0,
        minimum: 0.0,
        owner_id: "".to_string(),
        host: "".to_string(),
        status: LegacyFederationReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_federation_reservation("code-60", "name-748");
        assert_eq!(item.status, LegacyFederationReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_legacy_federation_reservation("code-60", "name-748");
        if item.move_to(LegacyFederationReservationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFederationReservationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_federation_reservation("code-60", "name-748");
        item.status = LegacyFederationReservationStatus::Stopped;
        assert!(
            item.move_to(LegacyFederationReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
