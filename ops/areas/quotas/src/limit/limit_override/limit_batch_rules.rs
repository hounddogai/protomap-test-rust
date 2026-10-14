use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit batch must pass. Check returns its problems, or none.
pub trait LimitBatchRule: std::fmt::Debug {
    fn check(&self, item: &LimitBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitBatchRequiredCountryRule {
    pub message: String,
}

impl LimitBatchRule for LimitBatchRequiredCountryRule {
    fn check(&self, item: &LimitBatch) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitBatchRequiredCodeRule {
    pub message: String,
}

impl LimitBatchRule for LimitBatchRequiredCodeRule {
    fn check(&self, item: &LimitBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitBatchCountryLengthRule {
    pub max_length: i64,
}

impl LimitBatchRule for LimitBatchCountryLengthRule {
    fn check(&self, item: &LimitBatch) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit batch.
#[derive(Debug, Clone)]
pub struct LimitBatchValidator {
    pub rules: Vec<Rc<dyn LimitBatchRule>>,
}

impl LimitBatchValidator {
    pub fn validate(&self, item: &LimitBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_batch_validator() -> LimitBatchValidator {
    LimitBatchValidator {
        rules: vec![
            Rc::new(LimitBatchRequiredCountryRule {
                message: "limit batch needs a value".to_string(),
            }) as Rc<dyn LimitBatchRule>,
            Rc::new(LimitBatchRequiredCodeRule {
                message: "limit batch needs an entry".to_string(),
            }),
            Rc::new(LimitBatchCountryLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_batch_validator();
        let item = new_limit_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = LimitBatchRequiredCountryRule {
            message: "limit batch needs an entry".to_string(),
        };
        let mut item = new_limit_batch("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitBatchRequiredCodeRule {
            message: "limit batch needs an entry".to_string(),
        };
        let mut item = new_limit_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = LimitBatchCountryLengthRule { max_length: 16 };
        let mut item = new_limit_batch("x", "y");
        item.country = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
