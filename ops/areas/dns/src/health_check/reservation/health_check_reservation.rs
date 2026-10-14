use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a health check reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl HealthCheckReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for HealthCheckReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReservation {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub attributes: BTreeMap<String, String>,
    pub capacity: i64,
    pub status: HealthCheckReservationStatus,
}

impl HealthCheckReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckReservationStatus) -> bool {
        match self.status {
            HealthCheckReservationStatus::Created => {
                return next == HealthCheckReservationStatus::Validated;
            }
            HealthCheckReservationStatus::Validated => {
                match next {
                    HealthCheckReservationStatus::Published | HealthCheckReservationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckReservationStatus::Published => {
                return next == HealthCheckReservationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> HealthCheckReservation {
        HealthCheckReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            attributes: self.attributes.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a health check reservation in status created.
pub fn new_health_check_reservation(code: &str, name: &str) -> HealthCheckReservation {
    HealthCheckReservation {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        attributes: BTreeMap::new(),
        capacity: 0,
        status: HealthCheckReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_health_check_reservation("code-491", "name-898");
        assert_eq!(item.status, HealthCheckReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_health_check_reservation("code-491", "name-898");
        if item.move_to(HealthCheckReservationStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckReservationStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_health_check_reservation("code-491", "name-898");
        item.status = HealthCheckReservationStatus::Retired;
        assert!(
            item.move_to(HealthCheckReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
