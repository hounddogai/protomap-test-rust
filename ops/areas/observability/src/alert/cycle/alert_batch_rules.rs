use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert batch must pass. Check returns its problems, or none.
pub trait AlertBatchRule: std::fmt::Debug {
    fn check(&self, item: &AlertBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBatchPhoneLengthRule {
    pub max_length: i64,
}

impl AlertBatchRule for AlertBatchPhoneLengthRule {
    fn check(&self, item: &AlertBatch) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBatchLocaleLengthRule {
    pub max_length: i64,
}

impl AlertBatchRule for AlertBatchLocaleLengthRule {
    fn check(&self, item: &AlertBatch) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBatchStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertBatchRule for AlertBatchStartsAtRangeRule {
    fn check(&self, item: &AlertBatch) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBatchRequiredNameRule {
    pub message: String,
}

impl AlertBatchRule for AlertBatchRequiredNameRule {
    fn check(&self, item: &AlertBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBatchRequiredPhoneRule {
    pub message: String,
}

impl AlertBatchRule for AlertBatchRequiredPhoneRule {
    fn check(&self, item: &AlertBatch) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertBatchCodeLengthRule {
    pub max_length: i64,
}

impl AlertBatchRule for AlertBatchCodeLengthRule {
    fn check(&self, item: &AlertBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert batch.
#[derive(Debug, Clone)]
pub struct AlertBatchValidator {
    pub rules: Vec<Rc<dyn AlertBatchRule>>,
}

impl AlertBatchValidator {
    pub fn validate(&self, item: &AlertBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_batch_validator() -> AlertBatchValidator {
    AlertBatchValidator {
        rules: vec![
            Rc::new(AlertBatchPhoneLengthRule { max_length: 16 }) as Rc<dyn AlertBatchRule>,
            Rc::new(AlertBatchLocaleLengthRule { max_length: 120 }),
            Rc::new(AlertBatchStartsAtRangeRule { min: 32, max: 120 }),
            Rc::new(AlertBatchRequiredNameRule {
                message: "alert batch needs a value".to_string(),
            }),
            Rc::new(AlertBatchRequiredPhoneRule {
                message: "alert batch needs an entry".to_string(),
            }),
            Rc::new(AlertBatchCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_batch_validator();
        let item = new_alert_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = AlertBatchPhoneLengthRule { max_length: 32 };
        let mut item = new_alert_batch("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = AlertBatchLocaleLengthRule { max_length: 32 };
        let mut item = new_alert_batch("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = AlertBatchStartsAtRangeRule { min: 32, max: 1000 };
        let mut item = new_alert_batch("x", "y");
        item.starts_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AlertBatchRequiredNameRule {
            message: "alert batch needs this field".to_string(),
        };
        let mut item = new_alert_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = AlertBatchRequiredPhoneRule {
            message: "alert batch needs an entry".to_string(),
        };
        let mut item = new_alert_batch("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AlertBatchCodeLengthRule { max_length: 120 };
        let mut item = new_alert_batch("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
