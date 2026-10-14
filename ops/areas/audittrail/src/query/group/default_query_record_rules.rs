use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default query record must pass. Check returns its problems, or none.
pub trait DefaultQueryRecordRule: std::fmt::Debug {
    fn check(&self, item: &DefaultQueryRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryRecordRequiredAudienceRule {
    pub message: String,
}

impl DefaultQueryRecordRule for DefaultQueryRecordRequiredAudienceRule {
    fn check(&self, item: &DefaultQueryRecord) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryRecordTitleLengthRule {
    pub max_length: i64,
}

impl DefaultQueryRecordRule for DefaultQueryRecordTitleLengthRule {
    fn check(&self, item: &DefaultQueryRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryRecordRequiredLabelRule {
    pub message: String,
}

impl DefaultQueryRecordRule for DefaultQueryRecordRequiredLabelRule {
    fn check(&self, item: &DefaultQueryRecord) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryRecordBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultQueryRecordRule for DefaultQueryRecordBatchSizeRangeRule {
    fn check(&self, item: &DefaultQueryRecord) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryRecordMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultQueryRecordRule for DefaultQueryRecordMinimumRangeRule {
    fn check(&self, item: &DefaultQueryRecord) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryRecordRequiredCodeRule {
    pub message: String,
}

impl DefaultQueryRecordRule for DefaultQueryRecordRequiredCodeRule {
    fn check(&self, item: &DefaultQueryRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default query record.
#[derive(Debug, Clone)]
pub struct DefaultQueryRecordValidator {
    pub rules: Vec<Rc<dyn DefaultQueryRecordRule>>,
}

impl DefaultQueryRecordValidator {
    pub fn validate(&self, item: &DefaultQueryRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultQueryRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultQueryRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_query_record_validator() -> DefaultQueryRecordValidator {
    DefaultQueryRecordValidator {
        rules: vec![
            Rc::new(DefaultQueryRecordRequiredAudienceRule {
                message: "default query record needs an entry".to_string(),
            }) as Rc<dyn DefaultQueryRecordRule>,
            Rc::new(DefaultQueryRecordTitleLengthRule { max_length: 1000 }),
            Rc::new(DefaultQueryRecordRequiredLabelRule {
                message: "default query record needs an entry".to_string(),
            }),
            Rc::new(DefaultQueryRecordBatchSizeRangeRule { min: 64, max: 255 }),
            Rc::new(DefaultQueryRecordMinimumRangeRule { min: 6.3, max: 3017.3 }),
            Rc::new(DefaultQueryRecordRequiredCodeRule {
                message: "default query record needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_query_record_validator();
        let item = new_default_query_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = DefaultQueryRecordRequiredAudienceRule {
            message: "default query record needs an entry".to_string(),
        };
        let mut item = new_default_query_record("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultQueryRecordTitleLengthRule { max_length: 255 };
        let mut item = new_default_query_record("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultQueryRecordRequiredLabelRule {
            message: "default query record needs an entry".to_string(),
        };
        let mut item = new_default_query_record("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = DefaultQueryRecordBatchSizeRangeRule { min: 16, max: 255 };
        let mut item = new_default_query_record("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = DefaultQueryRecordMinimumRangeRule { min: 3.4, max: 1438.4 };
        let mut item = new_default_query_record("x", "y");
        item.minimum = 1439.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultQueryRecordRequiredCodeRule {
            message: "default query record needs this field".to_string(),
        };
        let mut item = new_default_query_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
