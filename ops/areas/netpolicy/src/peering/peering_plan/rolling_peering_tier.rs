use super::*;

use acme_kit::text;

/// Where a rolling peering tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPeeringTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingPeeringTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingPeeringTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling peering tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringTier {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub description: String,
    pub attempts: i64,
    pub unit: String,
    pub phone: String,
    pub floor_value: f64,
    pub maximum: f64,
    pub created_at: i64,
    pub status: RollingPeeringTierStatus,
}

impl RollingPeeringTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPeeringTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPeeringTierStatus) -> bool {
        match self.status {
            RollingPeeringTierStatus::Proposed => {
                match next {
                    RollingPeeringTierStatus::Approved | RollingPeeringTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPeeringTierStatus::Approved => {
                return next == RollingPeeringTierStatus::Rejected || next == RollingPeeringTierStatus::Withdrawn;
            }
            RollingPeeringTierStatus::Rejected => {
                return next == RollingPeeringTierStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPeeringTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling peering tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.attempts as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPeeringTier {
        RollingPeeringTier {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            description: self.description.clone(),
            attempts: self.attempts,
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            floor_value: self.floor_value,
            maximum: self.maximum,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling peering tier in status proposed.
pub fn new_rolling_peering_tier(code: &str, name: &str) -> RollingPeeringTier {
    RollingPeeringTier {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        description: "".to_string(),
        attempts: 0,
        unit: "".to_string(),
        phone: "".to_string(),
        floor_value: 0.0,
        maximum: 0.0,
        created_at: 0,
        status: RollingPeeringTierStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_peering_tier("code-358", "name-782");
        assert_eq!(item.status, RollingPeeringTierStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_peering_tier("code-358", "name-782");
        if item.move_to(RollingPeeringTierStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPeeringTierStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_peering_tier("code-358", "name-782");
        item.status = RollingPeeringTierStatus::Withdrawn;
        assert!(
            item.move_to(RollingPeeringTierStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
