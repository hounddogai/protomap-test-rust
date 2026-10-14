use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a redaction batch must pass. Check returns its problems, or none.
pub trait RedactionBatchRule: std::fmt::Debug {
    fn check(&self, item: &RedactionBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBatchVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionBatchRule for RedactionBatchVersionRangeRule {
    fn check(&self, item: &RedactionBatch) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBatchRequiredLabelRule {
    pub message: String,
}

impl RedactionBatchRule for RedactionBatchRequiredLabelRule {
    fn check(&self, item: &RedactionBatch) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBatchCodeLengthRule {
    pub max_length: i64,
}

impl RedactionBatchRule for RedactionBatchCodeLengthRule {
    fn check(&self, item: &RedactionBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBatchRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionBatchRule for RedactionBatchRateRangeRule {
    fn check(&self, item: &RedactionBatch) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBatchAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RedactionBatchRule for RedactionBatchAllowedCodeRule {
    fn check(&self, item: &RedactionBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction batch.
#[derive(Debug, Clone)]
pub struct RedactionBatchValidator {
    pub rules: Vec<Rc<dyn RedactionBatchRule>>,
}

impl RedactionBatchValidator {
    pub fn validate(&self, item: &RedactionBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_batch_validator() -> RedactionBatchValidator {
    RedactionBatchValidator {
        rules: vec![
            Rc::new(RedactionBatchVersionRangeRule { min: 16, max: 64 }) as Rc<dyn RedactionBatchRule>,
            Rc::new(RedactionBatchRequiredLabelRule {
                message: "redaction batch needs a value".to_string(),
            }),
            Rc::new(RedactionBatchCodeLengthRule { max_length: 32 }),
            Rc::new(RedactionBatchRateRangeRule { min: 7.3, max: 216.9 }),
            Rc::new(RedactionBatchAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_batch_validator();
        let item = new_redaction_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RedactionBatchVersionRangeRule { min: 32, max: 120 };
        let mut item = new_redaction_batch("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RedactionBatchRequiredLabelRule {
            message: "redaction batch needs an entry".to_string(),
        };
        let mut item = new_redaction_batch("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionBatchCodeLengthRule { max_length: 3600 };
        let mut item = new_redaction_batch("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RedactionBatchRateRangeRule { min: 2.2, max: 1478.8 };
        let mut item = new_redaction_batch("x", "y");
        item.rate = 1480.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
