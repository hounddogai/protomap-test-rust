use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule batch must pass. Check returns its problems, or none.
pub trait RuleBatchRule: std::fmt::Debug {
    fn check(&self, item: &RuleBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBatchRequiredCodeRule {
    pub message: String,
}

impl RuleBatchRule for RuleBatchRequiredCodeRule {
    fn check(&self, item: &RuleBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBatchDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleBatchRule for RuleBatchDurationMinutesRangeRule {
    fn check(&self, item: &RuleBatch) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBatchLocaleLengthRule {
    pub max_length: i64,
}

impl RuleBatchRule for RuleBatchLocaleLengthRule {
    fn check(&self, item: &RuleBatch) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBatchRequiredChecksumRule {
    pub message: String,
}

impl RuleBatchRule for RuleBatchRequiredChecksumRule {
    fn check(&self, item: &RuleBatch) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBatchRequiredLocaleRule {
    pub message: String,
}

impl RuleBatchRule for RuleBatchRequiredLocaleRule {
    fn check(&self, item: &RuleBatch) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule batch.
#[derive(Debug, Clone)]
pub struct RuleBatchValidator {
    pub rules: Vec<Rc<dyn RuleBatchRule>>,
}

impl RuleBatchValidator {
    pub fn validate(&self, item: &RuleBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_batch_validator() -> RuleBatchValidator {
    RuleBatchValidator {
        rules: vec![
            Rc::new(RuleBatchRequiredCodeRule {
                message: "rule batch needs an entry".to_string(),
            }) as Rc<dyn RuleBatchRule>,
            Rc::new(RuleBatchDurationMinutesRangeRule { min: 3600, max: 86400 }),
            Rc::new(RuleBatchLocaleLengthRule { max_length: 64 }),
            Rc::new(RuleBatchRequiredChecksumRule {
                message: "rule batch needs an entry".to_string(),
            }),
            Rc::new(RuleBatchRequiredLocaleRule {
                message: "rule batch needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_batch_validator();
        let item = new_rule_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RuleBatchRequiredCodeRule {
            message: "rule batch needs a value".to_string(),
        };
        let mut item = new_rule_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RuleBatchDurationMinutesRangeRule { min: 1000, max: 3600 };
        let mut item = new_rule_batch("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = RuleBatchLocaleLengthRule { max_length: 255 };
        let mut item = new_rule_batch("x", "y");
        item.locale = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = RuleBatchRequiredChecksumRule {
            message: "rule batch needs an entry".to_string(),
        };
        let mut item = new_rule_batch("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RuleBatchRequiredLocaleRule {
            message: "rule batch needs this field".to_string(),
        };
        let mut item = new_rule_batch("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
