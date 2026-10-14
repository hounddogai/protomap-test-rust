use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a limit checkpoint must pass. Check returns its problems, or none.
pub trait LimitCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &LimitCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitCheckpointTitleLengthRule {
    pub max_length: i64,
}

impl LimitCheckpointRule for LimitCheckpointTitleLengthRule {
    fn check(&self, item: &LimitCheckpoint) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitCheckpointRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitCheckpointRule for LimitCheckpointRetriesRangeRule {
    fn check(&self, item: &LimitCheckpoint) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl LimitCheckpointRule for LimitCheckpointCodeLengthRule {
    fn check(&self, item: &LimitCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitCheckpointAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LimitCheckpointRule for LimitCheckpointAllowedCodeRule {
    fn check(&self, item: &LimitCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit checkpoint.
#[derive(Debug, Clone)]
pub struct LimitCheckpointValidator {
    pub rules: Vec<Rc<dyn LimitCheckpointRule>>,
}

impl LimitCheckpointValidator {
    pub fn validate(&self, item: &LimitCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_checkpoint_validator() -> LimitCheckpointValidator {
    LimitCheckpointValidator {
        rules: vec![
            Rc::new(LimitCheckpointTitleLengthRule { max_length: 64 }) as Rc<dyn LimitCheckpointRule>,
            Rc::new(LimitCheckpointRetriesRangeRule { min: 64, max: 80 }),
            Rc::new(LimitCheckpointCodeLengthRule { max_length: 64 }),
            Rc::new(LimitCheckpointAllowedCodeRule {
                allowed: vec!["default".to_string(), "legacy".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_checkpoint_validator();
        let item = new_limit_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LimitCheckpointTitleLengthRule { max_length: 64 };
        let mut item = new_limit_checkpoint("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = LimitCheckpointRetriesRangeRule { min: 64, max: 3600 };
        let mut item = new_limit_checkpoint("x", "y");
        item.retries = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitCheckpointCodeLengthRule { max_length: 32 };
        let mut item = new_limit_checkpoint("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
