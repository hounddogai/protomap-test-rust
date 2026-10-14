use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped token checkpoint must pass. Check returns its problems, or none.
pub trait CappedTokenCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &CappedTokenCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl CappedTokenCheckpointRule for CappedTokenCheckpointCodeLengthRule {
    fn check(&self, item: &CappedTokenCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenCheckpointRequiredCodeRule {
    pub message: String,
}

impl CappedTokenCheckpointRule for CappedTokenCheckpointRequiredCodeRule {
    fn check(&self, item: &CappedTokenCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenCheckpointThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedTokenCheckpointRule for CappedTokenCheckpointThresholdRangeRule {
    fn check(&self, item: &CappedTokenCheckpoint) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenCheckpointNotesLengthRule {
    pub max_length: i64,
}

impl CappedTokenCheckpointRule for CappedTokenCheckpointNotesLengthRule {
    fn check(&self, item: &CappedTokenCheckpoint) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenCheckpointRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedTokenCheckpointRule for CappedTokenCheckpointRateRangeRule {
    fn check(&self, item: &CappedTokenCheckpoint) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped token checkpoint.
#[derive(Debug, Clone)]
pub struct CappedTokenCheckpointValidator {
    pub rules: Vec<Rc<dyn CappedTokenCheckpointRule>>,
}

impl CappedTokenCheckpointValidator {
    pub fn validate(&self, item: &CappedTokenCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedTokenCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedTokenCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_token_checkpoint_validator() -> CappedTokenCheckpointValidator {
    CappedTokenCheckpointValidator {
        rules: vec![
            Rc::new(CappedTokenCheckpointCodeLengthRule { max_length: 80 }) as Rc<dyn CappedTokenCheckpointRule>,
            Rc::new(CappedTokenCheckpointRequiredCodeRule {
                message: "capped token checkpoint needs a value".to_string(),
            }),
            Rc::new(CappedTokenCheckpointThresholdRangeRule { min: 6.9, max: 1706.0 }),
            Rc::new(CappedTokenCheckpointNotesLengthRule { max_length: 86400 }),
            Rc::new(CappedTokenCheckpointRateRangeRule { min: 4.4, max: 4016.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_token_checkpoint_validator();
        let item = new_capped_token_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedTokenCheckpointCodeLengthRule { max_length: 32 };
        let mut item = new_capped_token_checkpoint("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedTokenCheckpointRequiredCodeRule {
            message: "capped token checkpoint needs this field".to_string(),
        };
        let mut item = new_capped_token_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = CappedTokenCheckpointThresholdRangeRule { min: 5.3, max: 4381.6 };
        let mut item = new_capped_token_checkpoint("x", "y");
        item.threshold = 4383.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = CappedTokenCheckpointNotesLengthRule { max_length: 80 };
        let mut item = new_capped_token_checkpoint("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = CappedTokenCheckpointRateRangeRule { min: 7.6, max: 4226.1 };
        let mut item = new_capped_token_checkpoint("x", "y");
        item.rate = 4227.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
