use super::*;

use acme_kit::text;

/// Where a adaptive region pin rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRegionPinRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveRegionPinRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveRegionPinRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive region pin rule as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinRule {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub cost: f64,
    pub scope: String,
    pub country: String,
    pub tenant_id: String,
    pub revision: i64,
    pub timezone: String,
    pub floor_value: f64,
    pub rotation_days: i64,
    pub status: AdaptiveRegionPinRuleStatus,
}

impl AdaptiveRegionPinRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRegionPinRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRegionPinRuleStatus) -> bool {
        match self.status {
            AdaptiveRegionPinRuleStatus::Proposed => {
                match next {
                    AdaptiveRegionPinRuleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRegionPinRuleStatus::Approved => {
                return next == AdaptiveRegionPinRuleStatus::Rejected;
            }
            AdaptiveRegionPinRuleStatus::Rejected => {
                return next == AdaptiveRegionPinRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRegionPinRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive region pin rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        (self.floor_value * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRegionPinRule {
        AdaptiveRegionPinRule {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            scope: self.scope.clone(),
            country: self.country.clone(),
            tenant_id: self.tenant_id.clone(),
            revision: self.revision,
            timezone: self.timezone.clone(),
            floor_value: self.floor_value,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive region pin rule in status proposed.
pub fn new_adaptive_region_pin_rule(code: &str, name: &str) -> AdaptiveRegionPinRule {
    AdaptiveRegionPinRule {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        cost: 0.0,
        scope: "".to_string(),
        country: "".to_string(),
        tenant_id: "".to_string(),
        revision: 0,
        timezone: "".to_string(),
        floor_value: 0.0,
        rotation_days: 0,
        status: AdaptiveRegionPinRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_region_pin_rule("code-277", "name-221");
        assert_eq!(item.status, AdaptiveRegionPinRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_adaptive_region_pin_rule("code-277", "name-221");
        if item.move_to(AdaptiveRegionPinRuleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRegionPinRuleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_region_pin_rule("code-277", "name-221");
        item.status = AdaptiveRegionPinRuleStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveRegionPinRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
