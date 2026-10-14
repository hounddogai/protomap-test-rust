use super::*;

use acme_kit::text;

/// Where a reservation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationMappingStatus {
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

impl std::fmt::Display for ReservationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation mapping as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationMapping {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub discount: f64,
    pub cost: f64,
    pub status: ReservationMappingStatus,
}

impl ReservationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationMappingStatus) -> bool {
        match self.status {
            ReservationMappingStatus::Idle => {
                return next == ReservationMappingStatus::Warming || next == ReservationMappingStatus::Stopped;
            }
            ReservationMappingStatus::Warming => {
                return next == ReservationMappingStatus::Ready;
            }
            ReservationMappingStatus::Ready => {
                match next {
                    ReservationMappingStatus::Draining | ReservationMappingStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationMappingStatus::Draining => {
                return next == ReservationMappingStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation mapping {} cannot move from {} to {}",
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
        format!("{} {}", "reservation mapping", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.cost - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationMapping {
        ReservationMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            discount: self.discount,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a reservation mapping in status idle.
pub fn new_reservation_mapping(code: &str, name: &str) -> ReservationMapping {
    ReservationMapping {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        discount: 0.0,
        cost: 0.0,
        status: ReservationMappingStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_reservation_mapping("code-444", "name-307");
        assert_eq!(item.status, ReservationMappingStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_reservation_mapping("code-444", "name-307");
        if item.move_to(ReservationMappingStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationMappingStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_reservation_mapping("code-444", "name-307");
        item.status = ReservationMappingStatus::Stopped;
        assert!(
            item.move_to(ReservationMappingStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
