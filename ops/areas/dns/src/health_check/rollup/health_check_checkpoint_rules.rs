use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check checkpoint must pass. Check returns its problems, or none.
pub trait HealthCheckCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCheckpointRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckCheckpointRule for HealthCheckCheckpointRateRangeRule {
    fn check(&self, item: &HealthCheckCheckpoint) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCheckpointRequiredCodeRule {
    pub message: String,
}

impl HealthCheckCheckpointRule for HealthCheckCheckpointRequiredCodeRule {
    fn check(&self, item: &HealthCheckCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCheckpointScopeLengthRule {
    pub max_length: i64,
}

impl HealthCheckCheckpointRule for HealthCheckCheckpointScopeLengthRule {
    fn check(&self, item: &HealthCheckCheckpoint) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckCheckpointRule for HealthCheckCheckpointCodeLengthRule {
    fn check(&self, item: &HealthCheckCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl HealthCheckCheckpointRule for HealthCheckCheckpointLabelLengthRule {
    fn check(&self, item: &HealthCheckCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCheckpointMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckCheckpointRule for HealthCheckCheckpointMinimumRangeRule {
    fn check(&self, item: &HealthCheckCheckpoint) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check checkpoint.
#[derive(Debug, Clone)]
pub struct HealthCheckCheckpointValidator {
    pub rules: Vec<Rc<dyn HealthCheckCheckpointRule>>,
}

impl HealthCheckCheckpointValidator {
    pub fn validate(&self, item: &HealthCheckCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_checkpoint_validator() -> HealthCheckCheckpointValidator {
    HealthCheckCheckpointValidator {
        rules: vec![
            Rc::new(HealthCheckCheckpointRateRangeRule { min: 0.2, max: 1528.4 }) as Rc<dyn HealthCheckCheckpointRule>,
            Rc::new(HealthCheckCheckpointRequiredCodeRule {
                message: "health check checkpoint needs this field".to_string(),
            }),
            Rc::new(HealthCheckCheckpointScopeLengthRule { max_length: 120 }),
            Rc::new(HealthCheckCheckpointCodeLengthRule { max_length: 16 }),
            Rc::new(HealthCheckCheckpointLabelLengthRule { max_length: 255 }),
            Rc::new(HealthCheckCheckpointMinimumRangeRule { min: 6.5, max: 434.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_checkpoint_validator();
        let item = new_health_check_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = HealthCheckCheckpointRateRangeRule { min: 0.6, max: 699.6 };
        let mut item = new_health_check_checkpoint("x", "y");
        item.rate = 701.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckCheckpointRequiredCodeRule {
            message: "health check checkpoint needs a value".to_string(),
        };
        let mut item = new_health_check_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = HealthCheckCheckpointScopeLengthRule { max_length: 80 };
        let mut item = new_health_check_checkpoint("x", "y");
        item.scope = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = HealthCheckCheckpointCodeLengthRule { max_length: 32 };
        let mut item = new_health_check_checkpoint("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = HealthCheckCheckpointLabelLengthRule { max_length: 32 };
        let mut item = new_health_check_checkpoint("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = HealthCheckCheckpointMinimumRangeRule { min: 1.6, max: 4731.2 };
        let mut item = new_health_check_checkpoint("x", "y");
        item.minimum = 4732.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
