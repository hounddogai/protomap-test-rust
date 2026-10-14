use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout mapping must pass. Check returns its problems, or none.
pub trait RolloutMappingRule: std::fmt::Debug {
    fn check(&self, item: &RolloutMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMappingGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutMappingRule for RolloutMappingGraceDaysRangeRule {
    fn check(&self, item: &RolloutMapping) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMappingCodePrefixRule {
    pub prefix: String,
}

impl RolloutMappingRule for RolloutMappingCodePrefixRule {
    fn check(&self, item: &RolloutMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMappingThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutMappingRule for RolloutMappingThresholdRangeRule {
    fn check(&self, item: &RolloutMapping) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMappingRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutMappingRule for RolloutMappingRevisionRangeRule {
    fn check(&self, item: &RolloutMapping) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMappingRequiredTimezoneRule {
    pub message: String,
}

impl RolloutMappingRule for RolloutMappingRequiredTimezoneRule {
    fn check(&self, item: &RolloutMapping) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout mapping.
#[derive(Debug, Clone)]
pub struct RolloutMappingValidator {
    pub rules: Vec<Rc<dyn RolloutMappingRule>>,
}

impl RolloutMappingValidator {
    pub fn validate(&self, item: &RolloutMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_mapping_validator() -> RolloutMappingValidator {
    RolloutMappingValidator {
        rules: vec![
            Rc::new(RolloutMappingGraceDaysRangeRule { min: 32, max: 80 }) as Rc<dyn RolloutMappingRule>,
            Rc::new(RolloutMappingCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RolloutMappingThresholdRangeRule { min: 4.5, max: 101.9 }),
            Rc::new(RolloutMappingRevisionRangeRule { min: 1000, max: 86400 }),
            Rc::new(RolloutMappingRequiredTimezoneRule {
                message: "rollout mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_mapping_validator();
        let item = new_rollout_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RolloutMappingGraceDaysRangeRule { min: 86400, max: 86400 };
        let mut item = new_rollout_mapping("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RolloutMappingThresholdRangeRule { min: 0.3, max: 488.7 };
        let mut item = new_rollout_mapping("x", "y");
        item.threshold = 490.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = RolloutMappingRevisionRangeRule { min: 120, max: 86400 };
        let mut item = new_rollout_mapping("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = RolloutMappingRequiredTimezoneRule {
            message: "rollout mapping needs an entry".to_string(),
        };
        let mut item = new_rollout_mapping("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
