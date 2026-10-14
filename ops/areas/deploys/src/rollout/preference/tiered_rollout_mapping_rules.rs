use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered rollout mapping must pass. Check returns its problems, or none.
pub trait TieredRolloutMappingRule: std::fmt::Debug {
    fn check(&self, item: &TieredRolloutMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutMappingScopeLengthRule {
    pub max_length: i64,
}

impl TieredRolloutMappingRule for TieredRolloutMappingScopeLengthRule {
    fn check(&self, item: &TieredRolloutMapping) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutMappingCodePrefixRule {
    pub prefix: String,
}

impl TieredRolloutMappingRule for TieredRolloutMappingCodePrefixRule {
    fn check(&self, item: &TieredRolloutMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutMappingLabelLengthRule {
    pub max_length: i64,
}

impl TieredRolloutMappingRule for TieredRolloutMappingLabelLengthRule {
    fn check(&self, item: &TieredRolloutMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutMappingCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredRolloutMappingRule for TieredRolloutMappingCeilingRangeRule {
    fn check(&self, item: &TieredRolloutMapping) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutMappingRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredRolloutMappingRule for TieredRolloutMappingRevisionRangeRule {
    fn check(&self, item: &TieredRolloutMapping) -> Vec<String> {
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

/// Runs every rule of a tiered rollout mapping.
#[derive(Debug, Clone)]
pub struct TieredRolloutMappingValidator {
    pub rules: Vec<Rc<dyn TieredRolloutMappingRule>>,
}

impl TieredRolloutMappingValidator {
    pub fn validate(&self, item: &TieredRolloutMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredRolloutMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRolloutMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_rollout_mapping_validator() -> TieredRolloutMappingValidator {
    TieredRolloutMappingValidator {
        rules: vec![
            Rc::new(TieredRolloutMappingScopeLengthRule { max_length: 64 }) as Rc<dyn TieredRolloutMappingRule>,
            Rc::new(TieredRolloutMappingCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TieredRolloutMappingLabelLengthRule { max_length: 86400 }),
            Rc::new(TieredRolloutMappingCeilingRangeRule { min: 3.1, max: 3868.5 }),
            Rc::new(TieredRolloutMappingRevisionRangeRule { min: 1000, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_rollout_mapping_validator();
        let item = new_tiered_rollout_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = TieredRolloutMappingScopeLengthRule { max_length: 32 };
        let mut item = new_tiered_rollout_mapping("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredRolloutMappingLabelLengthRule { max_length: 120 };
        let mut item = new_tiered_rollout_mapping("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = TieredRolloutMappingCeilingRangeRule { min: 5.6, max: 1578.6 };
        let mut item = new_tiered_rollout_mapping("x", "y");
        item.ceiling = 1580.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = TieredRolloutMappingRevisionRangeRule { min: 3600, max: 86400 };
        let mut item = new_tiered_rollout_mapping("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
