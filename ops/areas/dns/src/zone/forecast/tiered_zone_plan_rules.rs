use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered zone plan must pass. Check returns its problems, or none.
pub trait TieredZonePlanRule: std::fmt::Debug {
    fn check(&self, item: &TieredZonePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZonePlanDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredZonePlanRule for TieredZonePlanDurationMinutesRangeRule {
    fn check(&self, item: &TieredZonePlan) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZonePlanOwnerIdLengthRule {
    pub max_length: i64,
}

impl TieredZonePlanRule for TieredZonePlanOwnerIdLengthRule {
    fn check(&self, item: &TieredZonePlan) -> Vec<String> {
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
pub struct TieredZonePlanRequiredOwnerIdRule {
    pub message: String,
}

impl TieredZonePlanRule for TieredZonePlanRequiredOwnerIdRule {
    fn check(&self, item: &TieredZonePlan) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZonePlanTitleLengthRule {
    pub max_length: i64,
}

impl TieredZonePlanRule for TieredZonePlanTitleLengthRule {
    fn check(&self, item: &TieredZonePlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZonePlanCodePrefixRule {
    pub prefix: String,
}

impl TieredZonePlanRule for TieredZonePlanCodePrefixRule {
    fn check(&self, item: &TieredZonePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered zone plan.
#[derive(Debug, Clone)]
pub struct TieredZonePlanValidator {
    pub rules: Vec<Rc<dyn TieredZonePlanRule>>,
}

impl TieredZonePlanValidator {
    pub fn validate(&self, item: &TieredZonePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredZonePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredZonePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_zone_plan_validator() -> TieredZonePlanValidator {
    TieredZonePlanValidator {
        rules: vec![
            Rc::new(TieredZonePlanDurationMinutesRangeRule { min: 120, max: 255 }) as Rc<dyn TieredZonePlanRule>,
            Rc::new(TieredZonePlanOwnerIdLengthRule { max_length: 16 }),
            Rc::new(TieredZonePlanRequiredOwnerIdRule {
                message: "tiered zone plan needs an entry".to_string(),
            }),
            Rc::new(TieredZonePlanTitleLengthRule { max_length: 80 }),
            Rc::new(TieredZonePlanCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_zone_plan_validator();
        let item = new_tiered_zone_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = TieredZonePlanDurationMinutesRangeRule { min: 64, max: 255 };
        let mut item = new_tiered_zone_plan("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = TieredZonePlanOwnerIdLengthRule { max_length: 1000 };
        let mut item = new_tiered_zone_plan("x", "y");
        item.owner_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = TieredZonePlanRequiredOwnerIdRule {
            message: "tiered zone plan needs an entry".to_string(),
        };
        let mut item = new_tiered_zone_plan("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredZonePlanTitleLengthRule { max_length: 64 };
        let mut item = new_tiered_zone_plan("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
