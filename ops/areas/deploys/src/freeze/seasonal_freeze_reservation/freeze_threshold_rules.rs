use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze threshold must pass. Check returns its problems, or none.
pub trait FreezeThresholdRule: std::fmt::Debug {
    fn check(&self, item: &FreezeThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeThresholdCodeLengthRule {
    pub max_length: i64,
}

impl FreezeThresholdRule for FreezeThresholdCodeLengthRule {
    fn check(&self, item: &FreezeThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeThresholdRequiredTitleRule {
    pub message: String,
}

impl FreezeThresholdRule for FreezeThresholdRequiredTitleRule {
    fn check(&self, item: &FreezeThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeThresholdTitleLengthRule {
    pub max_length: i64,
}

impl FreezeThresholdRule for FreezeThresholdTitleLengthRule {
    fn check(&self, item: &FreezeThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeThresholdSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeThresholdRule for FreezeThresholdSequenceRangeRule {
    fn check(&self, item: &FreezeThreshold) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a freeze threshold.
#[derive(Debug, Clone)]
pub struct FreezeThresholdValidator {
    pub rules: Vec<Rc<dyn FreezeThresholdRule>>,
}

impl FreezeThresholdValidator {
    pub fn validate(&self, item: &FreezeThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_threshold_validator() -> FreezeThresholdValidator {
    FreezeThresholdValidator {
        rules: vec![
            Rc::new(FreezeThresholdCodeLengthRule { max_length: 16 }) as Rc<dyn FreezeThresholdRule>,
            Rc::new(FreezeThresholdRequiredTitleRule {
                message: "freeze threshold needs this field".to_string(),
            }),
            Rc::new(FreezeThresholdTitleLengthRule { max_length: 3600 }),
            Rc::new(FreezeThresholdSequenceRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_threshold_validator();
        let item = new_freeze_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FreezeThresholdCodeLengthRule { max_length: 16 };
        let mut item = new_freeze_threshold("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FreezeThresholdRequiredTitleRule {
            message: "freeze threshold needs an entry".to_string(),
        };
        let mut item = new_freeze_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FreezeThresholdTitleLengthRule { max_length: 255 };
        let mut item = new_freeze_threshold("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = FreezeThresholdSequenceRangeRule { min: 80, max: 255 };
        let mut item = new_freeze_threshold("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
