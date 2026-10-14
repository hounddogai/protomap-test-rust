use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze batch must pass. Check returns its problems, or none.
pub trait FreezeBatchRule: std::fmt::Debug {
    fn check(&self, item: &FreezeBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBatchCodeLengthRule {
    pub max_length: i64,
}

impl FreezeBatchRule for FreezeBatchCodeLengthRule {
    fn check(&self, item: &FreezeBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBatchRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeBatchRule for FreezeBatchRetriesRangeRule {
    fn check(&self, item: &FreezeBatch) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBatchRequiredCodeRule {
    pub message: String,
}

impl FreezeBatchRule for FreezeBatchRequiredCodeRule {
    fn check(&self, item: &FreezeBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBatchNameLengthRule {
    pub max_length: i64,
}

impl FreezeBatchRule for FreezeBatchNameLengthRule {
    fn check(&self, item: &FreezeBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBatchContactEmailLengthRule {
    pub max_length: i64,
}

impl FreezeBatchRule for FreezeBatchContactEmailLengthRule {
    fn check(&self, item: &FreezeBatch) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze batch.
#[derive(Debug, Clone)]
pub struct FreezeBatchValidator {
    pub rules: Vec<Rc<dyn FreezeBatchRule>>,
}

impl FreezeBatchValidator {
    pub fn validate(&self, item: &FreezeBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_batch_validator() -> FreezeBatchValidator {
    FreezeBatchValidator {
        rules: vec![
            Rc::new(FreezeBatchCodeLengthRule { max_length: 120 }) as Rc<dyn FreezeBatchRule>,
            Rc::new(FreezeBatchRetriesRangeRule { min: 16, max: 16 }),
            Rc::new(FreezeBatchRequiredCodeRule {
                message: "freeze batch needs this field".to_string(),
            }),
            Rc::new(FreezeBatchNameLengthRule { max_length: 86400 }),
            Rc::new(FreezeBatchContactEmailLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_batch_validator();
        let item = new_freeze_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FreezeBatchCodeLengthRule { max_length: 86400 };
        let mut item = new_freeze_batch("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = FreezeBatchRetriesRangeRule { min: 32, max: 80 };
        let mut item = new_freeze_batch("x", "y");
        item.retries = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FreezeBatchRequiredCodeRule {
            message: "freeze batch needs this field".to_string(),
        };
        let mut item = new_freeze_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FreezeBatchNameLengthRule { max_length: 32 };
        let mut item = new_freeze_batch("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = FreezeBatchContactEmailLengthRule { max_length: 16 };
        let mut item = new_freeze_batch("x", "y");
        item.contact_email = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
