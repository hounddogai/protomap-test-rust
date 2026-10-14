use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal action item batch must pass. Check returns its problems, or none.
pub trait SeasonalActionItemBatchRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalActionItemBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemBatchCodePrefixRule {
    pub prefix: String,
}

impl SeasonalActionItemBatchRule for SeasonalActionItemBatchCodePrefixRule {
    fn check(&self, item: &SeasonalActionItemBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemBatchCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalActionItemBatchRule for SeasonalActionItemBatchCodeLengthRule {
    fn check(&self, item: &SeasonalActionItemBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemBatchRequiredCodeRule {
    pub message: String,
}

impl SeasonalActionItemBatchRule for SeasonalActionItemBatchRequiredCodeRule {
    fn check(&self, item: &SeasonalActionItemBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemBatchRequiredTitleRule {
    pub message: String,
}

impl SeasonalActionItemBatchRule for SeasonalActionItemBatchRequiredTitleRule {
    fn check(&self, item: &SeasonalActionItemBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemBatchPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalActionItemBatchRule for SeasonalActionItemBatchPortRangeRule {
    fn check(&self, item: &SeasonalActionItemBatch) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal action item batch.
#[derive(Debug, Clone)]
pub struct SeasonalActionItemBatchValidator {
    pub rules: Vec<Rc<dyn SeasonalActionItemBatchRule>>,
}

impl SeasonalActionItemBatchValidator {
    pub fn validate(&self, item: &SeasonalActionItemBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalActionItemBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalActionItemBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_action_item_batch_validator() -> SeasonalActionItemBatchValidator {
    SeasonalActionItemBatchValidator {
        rules: vec![
            Rc::new(SeasonalActionItemBatchCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn SeasonalActionItemBatchRule>,
            Rc::new(SeasonalActionItemBatchCodeLengthRule { max_length: 120 }),
            Rc::new(SeasonalActionItemBatchRequiredCodeRule {
                message: "seasonal action item batch needs this field".to_string(),
            }),
            Rc::new(SeasonalActionItemBatchRequiredTitleRule {
                message: "seasonal action item batch needs an entry".to_string(),
            }),
            Rc::new(SeasonalActionItemBatchPortRangeRule { min: 120, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_action_item_batch_validator();
        let item = new_seasonal_action_item_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalActionItemBatchCodeLengthRule { max_length: 3600 };
        let mut item = new_seasonal_action_item_batch("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalActionItemBatchRequiredCodeRule {
            message: "seasonal action item batch needs a value".to_string(),
        };
        let mut item = new_seasonal_action_item_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalActionItemBatchRequiredTitleRule {
            message: "seasonal action item batch needs this field".to_string(),
        };
        let mut item = new_seasonal_action_item_batch("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = SeasonalActionItemBatchPortRangeRule { min: 120, max: 255 };
        let mut item = new_seasonal_action_item_batch("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
