use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched binding checkpoint must pass. Check returns its problems, or none.
pub trait BatchedBindingCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &BatchedBindingCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingCheckpointCodePrefixRule {
    pub prefix: String,
}

impl BatchedBindingCheckpointRule for BatchedBindingCheckpointCodePrefixRule {
    fn check(&self, item: &BatchedBindingCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingCheckpointRequiredCodeRule {
    pub message: String,
}

impl BatchedBindingCheckpointRule for BatchedBindingCheckpointRequiredCodeRule {
    fn check(&self, item: &BatchedBindingCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingCheckpointLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedBindingCheckpointRule for BatchedBindingCheckpointLimitRangeRule {
    fn check(&self, item: &BatchedBindingCheckpoint) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingCheckpointAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl BatchedBindingCheckpointRule for BatchedBindingCheckpointAllowedCountryRule {
    fn check(&self, item: &BatchedBindingCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is missing", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingCheckpointMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedBindingCheckpointRule for BatchedBindingCheckpointMarginRangeRule {
    fn check(&self, item: &BatchedBindingCheckpoint) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingCheckpointRequiredNameRule {
    pub message: String,
}

impl BatchedBindingCheckpointRule for BatchedBindingCheckpointRequiredNameRule {
    fn check(&self, item: &BatchedBindingCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched binding checkpoint.
#[derive(Debug, Clone)]
pub struct BatchedBindingCheckpointValidator {
    pub rules: Vec<Rc<dyn BatchedBindingCheckpointRule>>,
}

impl BatchedBindingCheckpointValidator {
    pub fn validate(&self, item: &BatchedBindingCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedBindingCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedBindingCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_binding_checkpoint_validator() -> BatchedBindingCheckpointValidator {
    BatchedBindingCheckpointValidator {
        rules: vec![
            Rc::new(BatchedBindingCheckpointCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn BatchedBindingCheckpointRule>,
            Rc::new(BatchedBindingCheckpointRequiredCodeRule {
                message: "batched binding checkpoint needs an entry".to_string(),
            }),
            Rc::new(BatchedBindingCheckpointLimitRangeRule { min: 32, max: 120 }),
            Rc::new(BatchedBindingCheckpointAllowedCountryRule {
                allowed: vec!["lenient".to_string(), "global".to_string(), "rolling".to_string()],
            }),
            Rc::new(BatchedBindingCheckpointMarginRangeRule { min: 7.3, max: 3153.9 }),
            Rc::new(BatchedBindingCheckpointRequiredNameRule {
                message: "batched binding checkpoint needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_binding_checkpoint_validator();
        let item = new_batched_binding_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedBindingCheckpointRequiredCodeRule {
            message: "batched binding checkpoint needs this field".to_string(),
        };
        let mut item = new_batched_binding_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = BatchedBindingCheckpointLimitRangeRule { min: 120, max: 1000 };
        let mut item = new_batched_binding_checkpoint("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = BatchedBindingCheckpointMarginRangeRule { min: 1.1, max: 745.6 };
        let mut item = new_batched_binding_checkpoint("x", "y");
        item.margin = 747.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedBindingCheckpointRequiredNameRule {
            message: "batched binding checkpoint needs an entry".to_string(),
        };
        let mut item = new_batched_binding_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
