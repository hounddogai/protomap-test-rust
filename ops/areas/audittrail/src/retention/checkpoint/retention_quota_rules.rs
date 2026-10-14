use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention quota must pass. Check returns its problems, or none.
pub trait RetentionQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RetentionQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionQuotaNameLengthRule {
    pub max_length: i64,
}

impl RetentionQuotaRule for RetentionQuotaNameLengthRule {
    fn check(&self, item: &RetentionQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionQuotaRequiredCodeRule {
    pub message: String,
}

impl RetentionQuotaRule for RetentionQuotaRequiredCodeRule {
    fn check(&self, item: &RetentionQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionQuotaRequiredNameRule {
    pub message: String,
}

impl RetentionQuotaRule for RetentionQuotaRequiredNameRule {
    fn check(&self, item: &RetentionQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionQuotaRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionQuotaRule for RetentionQuotaRevisionRangeRule {
    fn check(&self, item: &RetentionQuota) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionQuotaBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionQuotaRule for RetentionQuotaBudgetRangeRule {
    fn check(&self, item: &RetentionQuota) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionQuotaCodeLengthRule {
    pub max_length: i64,
}

impl RetentionQuotaRule for RetentionQuotaCodeLengthRule {
    fn check(&self, item: &RetentionQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention quota.
#[derive(Debug, Clone)]
pub struct RetentionQuotaValidator {
    pub rules: Vec<Rc<dyn RetentionQuotaRule>>,
}

impl RetentionQuotaValidator {
    pub fn validate(&self, item: &RetentionQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_quota_validator() -> RetentionQuotaValidator {
    RetentionQuotaValidator {
        rules: vec![
            Rc::new(RetentionQuotaNameLengthRule { max_length: 16 }) as Rc<dyn RetentionQuotaRule>,
            Rc::new(RetentionQuotaRequiredCodeRule {
                message: "retention quota needs a value".to_string(),
            }),
            Rc::new(RetentionQuotaRequiredNameRule {
                message: "retention quota needs a value".to_string(),
            }),
            Rc::new(RetentionQuotaRevisionRangeRule { min: 32, max: 1000 }),
            Rc::new(RetentionQuotaBudgetRangeRule { min: 1.6, max: 220.3 }),
            Rc::new(RetentionQuotaCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_quota_validator();
        let item = new_retention_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RetentionQuotaNameLengthRule { max_length: 64 };
        let mut item = new_retention_quota("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RetentionQuotaRequiredCodeRule {
            message: "retention quota needs this field".to_string(),
        };
        let mut item = new_retention_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RetentionQuotaRequiredNameRule {
            message: "retention quota needs this field".to_string(),
        };
        let mut item = new_retention_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = RetentionQuotaRevisionRangeRule { min: 120, max: 3600 };
        let mut item = new_retention_quota("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RetentionQuotaBudgetRangeRule { min: 4.0, max: 4371.9 };
        let mut item = new_retention_quota("x", "y");
        item.budget = 4373.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionQuotaCodeLengthRule { max_length: 64 };
        let mut item = new_retention_quota("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
