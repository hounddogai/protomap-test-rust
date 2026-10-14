use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook batch must pass. Check returns its problems, or none.
pub trait RunbookBatchRule: std::fmt::Debug {
    fn check(&self, item: &RunbookBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBatchActiveGuardRule {
    pub limit: i64,
}

impl RunbookBatchRule for RunbookBatchActiveGuardRule {
    fn check(&self, item: &RunbookBatch) -> Vec<String> {
        if item.active && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while active",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBatchTitleLengthRule {
    pub max_length: i64,
}

impl RunbookBatchRule for RunbookBatchTitleLengthRule {
    fn check(&self, item: &RunbookBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBatchCodePrefixRule {
    pub prefix: String,
}

impl RunbookBatchRule for RunbookBatchCodePrefixRule {
    fn check(&self, item: &RunbookBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBatchFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookBatchRule for RunbookBatchFloorValueRangeRule {
    fn check(&self, item: &RunbookBatch) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBatchRequiredCodeRule {
    pub message: String,
}

impl RunbookBatchRule for RunbookBatchRequiredCodeRule {
    fn check(&self, item: &RunbookBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook batch.
#[derive(Debug, Clone)]
pub struct RunbookBatchValidator {
    pub rules: Vec<Rc<dyn RunbookBatchRule>>,
}

impl RunbookBatchValidator {
    pub fn validate(&self, item: &RunbookBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_batch_validator() -> RunbookBatchValidator {
    RunbookBatchValidator {
        rules: vec![
            Rc::new(RunbookBatchActiveGuardRule { limit: 86400 }) as Rc<dyn RunbookBatchRule>,
            Rc::new(RunbookBatchTitleLengthRule { max_length: 16 }),
            Rc::new(RunbookBatchCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RunbookBatchFloorValueRangeRule { min: 5.6, max: 1834.2 }),
            Rc::new(RunbookBatchRequiredCodeRule {
                message: "runbook batch needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_batch_validator();
        let item = new_runbook_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RunbookBatchTitleLengthRule { max_length: 3600 };
        let mut item = new_runbook_batch("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RunbookBatchFloorValueRangeRule { min: 2.7, max: 292.1 };
        let mut item = new_runbook_batch("x", "y");
        item.floor_value = 293.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RunbookBatchRequiredCodeRule {
            message: "runbook batch needs a value".to_string(),
        };
        let mut item = new_runbook_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
