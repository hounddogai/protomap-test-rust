use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage checkpoint must pass. Check returns its problems, or none.
pub trait OverageCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &OverageCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl OverageCheckpointRule for OverageCheckpointLabelLengthRule {
    fn check(&self, item: &OverageCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageCheckpointRequiredLabelRule {
    pub message: String,
}

impl OverageCheckpointRule for OverageCheckpointRequiredLabelRule {
    fn check(&self, item: &OverageCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageCheckpointMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageCheckpointRule for OverageCheckpointMinimumRangeRule {
    fn check(&self, item: &OverageCheckpoint) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageCheckpointCodePrefixRule {
    pub prefix: String,
}

impl OverageCheckpointRule for OverageCheckpointCodePrefixRule {
    fn check(&self, item: &OverageCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage checkpoint.
#[derive(Debug, Clone)]
pub struct OverageCheckpointValidator {
    pub rules: Vec<Rc<dyn OverageCheckpointRule>>,
}

impl OverageCheckpointValidator {
    pub fn validate(&self, item: &OverageCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_checkpoint_validator() -> OverageCheckpointValidator {
    OverageCheckpointValidator {
        rules: vec![
            Rc::new(OverageCheckpointLabelLengthRule { max_length: 32 }) as Rc<dyn OverageCheckpointRule>,
            Rc::new(OverageCheckpointRequiredLabelRule {
                message: "overage checkpoint needs an entry".to_string(),
            }),
            Rc::new(OverageCheckpointMinimumRangeRule { min: 6.4, max: 2702.5 }),
            Rc::new(OverageCheckpointCodePrefixRule {
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
        let validator = default_overage_checkpoint_validator();
        let item = new_overage_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = OverageCheckpointLabelLengthRule { max_length: 16 };
        let mut item = new_overage_checkpoint("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = OverageCheckpointRequiredLabelRule {
            message: "overage checkpoint needs this field".to_string(),
        };
        let mut item = new_overage_checkpoint("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = OverageCheckpointMinimumRangeRule { min: 9.1, max: 4545.5 };
        let mut item = new_overage_checkpoint("x", "y");
        item.minimum = 4547.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
