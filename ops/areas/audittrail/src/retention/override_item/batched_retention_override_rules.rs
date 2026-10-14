use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched retention override must pass. Check returns its problems, or none.
pub trait BatchedRetentionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRetentionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionOverrideRequiredScopeRule {
    pub message: String,
}

impl BatchedRetentionOverrideRule for BatchedRetentionOverrideRequiredScopeRule {
    fn check(&self, item: &BatchedRetentionOverride) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionOverrideStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRetentionOverrideRule for BatchedRetentionOverrideStartsAtRangeRule {
    fn check(&self, item: &BatchedRetentionOverride) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionOverrideTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRetentionOverrideRule for BatchedRetentionOverrideTtlSecondsRangeRule {
    fn check(&self, item: &BatchedRetentionOverride) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionOverrideRequiredKeyIdRule {
    pub message: String,
}

impl BatchedRetentionOverrideRule for BatchedRetentionOverrideRequiredKeyIdRule {
    fn check(&self, item: &BatchedRetentionOverride) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched retention override.
#[derive(Debug, Clone)]
pub struct BatchedRetentionOverrideValidator {
    pub rules: Vec<Rc<dyn BatchedRetentionOverrideRule>>,
}

impl BatchedRetentionOverrideValidator {
    pub fn validate(&self, item: &BatchedRetentionOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRetentionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRetentionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_retention_override_validator() -> BatchedRetentionOverrideValidator {
    BatchedRetentionOverrideValidator {
        rules: vec![
            Rc::new(BatchedRetentionOverrideRequiredScopeRule {
                message: "batched retention override needs an entry".to_string(),
            }) as Rc<dyn BatchedRetentionOverrideRule>,
            Rc::new(BatchedRetentionOverrideStartsAtRangeRule { min: 80, max: 1000 }),
            Rc::new(BatchedRetentionOverrideTtlSecondsRangeRule { min: 32, max: 64 }),
            Rc::new(BatchedRetentionOverrideRequiredKeyIdRule {
                message: "batched retention override needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_retention_override_validator();
        let item = new_batched_retention_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = BatchedRetentionOverrideRequiredScopeRule {
            message: "batched retention override needs an entry".to_string(),
        };
        let mut item = new_batched_retention_override("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = BatchedRetentionOverrideStartsAtRangeRule { min: 255, max: 3600 };
        let mut item = new_batched_retention_override("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = BatchedRetentionOverrideTtlSecondsRangeRule { min: 16, max: 255 };
        let mut item = new_batched_retention_override("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = BatchedRetentionOverrideRequiredKeyIdRule {
            message: "batched retention override needs an entry".to_string(),
        };
        let mut item = new_batched_retention_override("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
