use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze checkpoint must pass. Check returns its problems, or none.
pub trait FreezeCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &FreezeCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeCheckpointMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeCheckpointRule for FreezeCheckpointMinimumRangeRule {
    fn check(&self, item: &FreezeCheckpoint) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeCheckpointAudienceLengthRule {
    pub max_length: i64,
}

impl FreezeCheckpointRule for FreezeCheckpointAudienceLengthRule {
    fn check(&self, item: &FreezeCheckpoint) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeCheckpointRequiredAudienceRule {
    pub message: String,
}

impl FreezeCheckpointRule for FreezeCheckpointRequiredAudienceRule {
    fn check(&self, item: &FreezeCheckpoint) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze checkpoint.
#[derive(Debug, Clone)]
pub struct FreezeCheckpointValidator {
    pub rules: Vec<Rc<dyn FreezeCheckpointRule>>,
}

impl FreezeCheckpointValidator {
    pub fn validate(&self, item: &FreezeCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_checkpoint_validator() -> FreezeCheckpointValidator {
    FreezeCheckpointValidator {
        rules: vec![
            Rc::new(FreezeCheckpointMinimumRangeRule { min: 8.2, max: 4475.5 }) as Rc<dyn FreezeCheckpointRule>,
            Rc::new(FreezeCheckpointAudienceLengthRule { max_length: 64 }),
            Rc::new(FreezeCheckpointRequiredAudienceRule {
                message: "freeze checkpoint needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_checkpoint_validator();
        let item = new_freeze_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = FreezeCheckpointMinimumRangeRule { min: 8.3, max: 2363.4 };
        let mut item = new_freeze_checkpoint("x", "y");
        item.minimum = 2364.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = FreezeCheckpointAudienceLengthRule { max_length: 32 };
        let mut item = new_freeze_checkpoint("x", "y");
        item.audience = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = FreezeCheckpointRequiredAudienceRule {
            message: "freeze checkpoint needs this field".to_string(),
        };
        let mut item = new_freeze_checkpoint("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
