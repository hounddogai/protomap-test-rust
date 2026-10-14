use super::*;

use acme_kit::text;

/// Where a rolling peering override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPeeringOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingPeeringOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingPeeringOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling peering override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringOverride {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub attempts: i64,
    pub ttl_seconds: i64,
    pub status: RollingPeeringOverrideStatus,
}

impl RollingPeeringOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPeeringOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPeeringOverrideStatus) -> bool {
        match self.status {
            RollingPeeringOverrideStatus::Created => {
                return next == RollingPeeringOverrideStatus::Retired;
            }
            RollingPeeringOverrideStatus::Validated => {
                match next {
                    RollingPeeringOverrideStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPeeringOverrideStatus::Published => {
                return next == RollingPeeringOverrideStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPeeringOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling peering override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPeeringOverride {
        RollingPeeringOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            attempts: self.attempts,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling peering override in status created.
pub fn new_rolling_peering_override(code: &str, name: &str) -> RollingPeeringOverride {
    RollingPeeringOverride {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        attempts: 0,
        ttl_seconds: 0,
        status: RollingPeeringOverrideStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rolling_peering_override("code-180", "name-876");
        assert_eq!(item.status, RollingPeeringOverrideStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_rolling_peering_override("code-180", "name-876");
        if item.move_to(RollingPeeringOverrideStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPeeringOverrideStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rolling_peering_override("code-180", "name-876");
        item.status = RollingPeeringOverrideStatus::Retired;
        assert!(
            item.move_to(RollingPeeringOverrideStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
