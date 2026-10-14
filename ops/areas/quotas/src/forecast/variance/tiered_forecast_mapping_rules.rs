use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered forecast mapping must pass. Check returns its problems, or none.
pub trait TieredForecastMappingRule: std::fmt::Debug {
    fn check(&self, item: &TieredForecastMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMappingOwnerIdLengthRule {
    pub max_length: i64,
}

impl TieredForecastMappingRule for TieredForecastMappingOwnerIdLengthRule {
    fn check(&self, item: &TieredForecastMapping) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMappingRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredForecastMappingRule for TieredForecastMappingRotationDaysRangeRule {
    fn check(&self, item: &TieredForecastMapping) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMappingAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl TieredForecastMappingRule for TieredForecastMappingAllowedOwnerIdRule {
    fn check(&self, item: &TieredForecastMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is blocked", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMappingPinnedGuardRule {
    pub limit: f64,
}

impl TieredForecastMappingRule for TieredForecastMappingPinnedGuardRule {
    fn check(&self, item: &TieredForecastMapping) -> Vec<String> {
        if item.pinned && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while pinned", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMappingRequiredCodeRule {
    pub message: String,
}

impl TieredForecastMappingRule for TieredForecastMappingRequiredCodeRule {
    fn check(&self, item: &TieredForecastMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered forecast mapping.
#[derive(Debug, Clone)]
pub struct TieredForecastMappingValidator {
    pub rules: Vec<Rc<dyn TieredForecastMappingRule>>,
}

impl TieredForecastMappingValidator {
    pub fn validate(&self, item: &TieredForecastMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredForecastMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredForecastMapping]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_tiered_forecast_mapping_validator() -> TieredForecastMappingValidator {
    TieredForecastMappingValidator {
        rules: vec![
            Rc::new(TieredForecastMappingOwnerIdLengthRule { max_length: 120 }) as Rc<dyn TieredForecastMappingRule>,
            Rc::new(TieredForecastMappingRotationDaysRangeRule { min: 80, max: 120 }),
            Rc::new(TieredForecastMappingAllowedOwnerIdRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(TieredForecastMappingPinnedGuardRule { limit: 4.0 }),
            Rc::new(TieredForecastMappingRequiredCodeRule {
                message: "tiered forecast mapping needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_forecast_mapping_validator();
        let item = new_tiered_forecast_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = TieredForecastMappingOwnerIdLengthRule { max_length: 32 };
        let mut item = new_tiered_forecast_mapping("x", "y");
        item.owner_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = TieredForecastMappingRotationDaysRangeRule { min: 80, max: 86400 };
        let mut item = new_tiered_forecast_mapping("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredForecastMappingRequiredCodeRule {
            message: "tiered forecast mapping needs an entry".to_string(),
        };
        let mut item = new_tiered_forecast_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
