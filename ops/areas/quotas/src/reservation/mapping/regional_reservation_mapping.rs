use super::*;

use acme_kit::text;

/// Where a regional reservation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalReservationMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalReservationMappingStatus {
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

impl std::fmt::Display for RegionalReservationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional reservation mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationMapping {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub tenant_id: String,
    pub port: i64,
    pub weight: f64,
    pub namespace: String,
    pub ttl_seconds: i64,
    pub cost: f64,
    pub key_id: String,
    pub status: RegionalReservationMappingStatus,
}

impl RegionalReservationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalReservationMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalReservationMappingStatus) -> bool {
        match self.status {
            RegionalReservationMappingStatus::Idle => {
                match next {
                    RegionalReservationMappingStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalReservationMappingStatus::Warming => {
                return next == RegionalReservationMappingStatus::Draining
                    || next == RegionalReservationMappingStatus::Stopped;
            }
            RegionalReservationMappingStatus::Ready => {
                match next {
                    RegionalReservationMappingStatus::Draining | RegionalReservationMappingStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalReservationMappingStatus::Draining => {
                match next {
                    RegionalReservationMappingStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalReservationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional reservation mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.maximum + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalReservationMapping {
        RegionalReservationMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            weight: self.weight,
            namespace: self.namespace.clone(),
            ttl_seconds: self.ttl_seconds,
            cost: self.cost,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional reservation mapping in status idle.
pub fn new_regional_reservation_mapping(code: &str, name: &str) -> RegionalReservationMapping {
    RegionalReservationMapping {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        tenant_id: "".to_string(),
        port: 0,
        weight: 0.0,
        namespace: "".to_string(),
        ttl_seconds: 0,
        cost: 0.0,
        key_id: "".to_string(),
        status: RegionalReservationMappingStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_reservation_mapping("code-479", "name-291");
        assert_eq!(item.status, RegionalReservationMappingStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_regional_reservation_mapping("code-479", "name-291");
        if item.move_to(RegionalReservationMappingStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalReservationMappingStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_reservation_mapping("code-479", "name-291");
        item.status = RegionalReservationMappingStatus::Stopped;
        assert!(
            item.move_to(RegionalReservationMappingStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
