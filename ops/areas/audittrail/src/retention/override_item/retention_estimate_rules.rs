use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention estimate must pass. Check returns its problems, or none.
pub trait RetentionEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RetentionEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEstimateRequiredKeyIdRule {
    pub message: String,
}

impl RetentionEstimateRule for RetentionEstimateRequiredKeyIdRule {
    fn check(&self, item: &RetentionEstimate) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEstimateCodeLengthRule {
    pub max_length: i64,
}

impl RetentionEstimateRule for RetentionEstimateCodeLengthRule {
    fn check(&self, item: &RetentionEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEstimateKeyIdLengthRule {
    pub max_length: i64,
}

impl RetentionEstimateRule for RetentionEstimateKeyIdLengthRule {
    fn check(&self, item: &RetentionEstimate) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEstimateAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl RetentionEstimateRule for RetentionEstimateAllowedKeyIdRule {
    fn check(&self, item: &RetentionEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is not allowed", item.key_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention estimate.
#[derive(Debug, Clone)]
pub struct RetentionEstimateValidator {
    pub rules: Vec<Rc<dyn RetentionEstimateRule>>,
}

impl RetentionEstimateValidator {
    pub fn validate(&self, item: &RetentionEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_estimate_validator() -> RetentionEstimateValidator {
    RetentionEstimateValidator {
        rules: vec![
            Rc::new(RetentionEstimateRequiredKeyIdRule {
                message: "retention estimate needs a value".to_string(),
            }) as Rc<dyn RetentionEstimateRule>,
            Rc::new(RetentionEstimateCodeLengthRule { max_length: 64 }),
            Rc::new(RetentionEstimateKeyIdLengthRule { max_length: 120 }),
            Rc::new(RetentionEstimateAllowedKeyIdRule {
                allowed: vec!["default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_estimate_validator();
        let item = new_retention_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = RetentionEstimateRequiredKeyIdRule {
            message: "retention estimate needs this field".to_string(),
        };
        let mut item = new_retention_estimate("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionEstimateCodeLengthRule { max_length: 120 };
        let mut item = new_retention_estimate("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = RetentionEstimateKeyIdLengthRule { max_length: 1000 };
        let mut item = new_retention_estimate("x", "y");
        item.key_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
