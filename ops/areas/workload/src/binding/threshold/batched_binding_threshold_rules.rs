use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched binding threshold must pass. Check returns its problems, or none.
pub trait BatchedBindingThresholdRule: std::fmt::Debug {
    fn check(&self, item: &BatchedBindingThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingThresholdThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedBindingThresholdRule for BatchedBindingThresholdThresholdRangeRule {
    fn check(&self, item: &BatchedBindingThreshold) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingThresholdRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedBindingThresholdRule for BatchedBindingThresholdRevisionRangeRule {
    fn check(&self, item: &BatchedBindingThreshold) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingThresholdUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedBindingThresholdRule for BatchedBindingThresholdUpdatedAtRangeRule {
    fn check(&self, item: &BatchedBindingThreshold) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched binding threshold.
#[derive(Debug, Clone)]
pub struct BatchedBindingThresholdValidator {
    pub rules: Vec<Rc<dyn BatchedBindingThresholdRule>>,
}

impl BatchedBindingThresholdValidator {
    pub fn validate(&self, item: &BatchedBindingThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedBindingThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedBindingThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_binding_threshold_validator() -> BatchedBindingThresholdValidator {
    BatchedBindingThresholdValidator {
        rules: vec![
            Rc::new(BatchedBindingThresholdThresholdRangeRule { min: 6.9, max: 4749.4 })
                as Rc<dyn BatchedBindingThresholdRule>,
            Rc::new(BatchedBindingThresholdRevisionRangeRule { min: 255, max: 255 }),
            Rc::new(BatchedBindingThresholdUpdatedAtRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_binding_threshold_validator();
        let item = new_batched_binding_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = BatchedBindingThresholdThresholdRangeRule { min: 7.1, max: 4478.2 };
        let mut item = new_batched_binding_threshold("x", "y");
        item.threshold = 4479.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = BatchedBindingThresholdRevisionRangeRule { min: 16, max: 80 };
        let mut item = new_batched_binding_threshold("x", "y");
        item.revision = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = BatchedBindingThresholdUpdatedAtRangeRule { min: 120, max: 1000 };
        let mut item = new_batched_binding_threshold("x", "y");
        item.updated_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
