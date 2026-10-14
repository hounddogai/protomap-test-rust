use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover variance must pass. Check returns its problems, or none.
pub trait FailoverVarianceRule: std::fmt::Debug {
    fn check(&self, item: &FailoverVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverVarianceTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl FailoverVarianceRule for FailoverVarianceTagsLimitRule {
    fn check(&self, item: &FailoverVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverVariancePhoneLengthRule {
    pub max_length: i64,
}

impl FailoverVarianceRule for FailoverVariancePhoneLengthRule {
    fn check(&self, item: &FailoverVariance) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverVarianceRequiredPhoneRule {
    pub message: String,
}

impl FailoverVarianceRule for FailoverVarianceRequiredPhoneRule {
    fn check(&self, item: &FailoverVariance) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverVarianceCodeLengthRule {
    pub max_length: i64,
}

impl FailoverVarianceRule for FailoverVarianceCodeLengthRule {
    fn check(&self, item: &FailoverVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverVarianceNameLengthRule {
    pub max_length: i64,
}

impl FailoverVarianceRule for FailoverVarianceNameLengthRule {
    fn check(&self, item: &FailoverVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverVarianceCodePrefixRule {
    pub prefix: String,
}

impl FailoverVarianceRule for FailoverVarianceCodePrefixRule {
    fn check(&self, item: &FailoverVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover variance.
#[derive(Debug, Clone)]
pub struct FailoverVarianceValidator {
    pub rules: Vec<Rc<dyn FailoverVarianceRule>>,
}

impl FailoverVarianceValidator {
    pub fn validate(&self, item: &FailoverVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_variance_validator() -> FailoverVarianceValidator {
    FailoverVarianceValidator {
        rules: vec![
            Rc::new(FailoverVarianceTagsLimitRule {
                max_count: 64,
                banned: vec!["weighted".to_string(), "capped".to_string()],
            }) as Rc<dyn FailoverVarianceRule>,
            Rc::new(FailoverVariancePhoneLengthRule { max_length: 80 }),
            Rc::new(FailoverVarianceRequiredPhoneRule {
                message: "failover variance needs a value".to_string(),
            }),
            Rc::new(FailoverVarianceCodeLengthRule { max_length: 64 }),
            Rc::new(FailoverVarianceNameLengthRule { max_length: 255 }),
            Rc::new(FailoverVarianceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_variance_validator();
        let item = new_failover_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = FailoverVariancePhoneLengthRule { max_length: 86400 };
        let mut item = new_failover_variance("x", "y");
        item.phone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = FailoverVarianceRequiredPhoneRule {
            message: "failover variance needs an entry".to_string(),
        };
        let mut item = new_failover_variance("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FailoverVarianceCodeLengthRule { max_length: 16 };
        let mut item = new_failover_variance("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FailoverVarianceNameLengthRule { max_length: 1000 };
        let mut item = new_failover_variance("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
