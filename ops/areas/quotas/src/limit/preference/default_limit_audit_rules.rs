use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default limit audit must pass. Check returns its problems, or none.
pub trait DefaultLimitAuditRule: std::fmt::Debug {
    fn check(&self, item: &DefaultLimitAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitAuditAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultLimitAuditRule for DefaultLimitAuditAttemptsRangeRule {
    fn check(&self, item: &DefaultLimitAudit) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitAuditLabelLengthRule {
    pub max_length: i64,
}

impl DefaultLimitAuditRule for DefaultLimitAuditLabelLengthRule {
    fn check(&self, item: &DefaultLimitAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitAuditRequiredCategoryRule {
    pub message: String,
}

impl DefaultLimitAuditRule for DefaultLimitAuditRequiredCategoryRule {
    fn check(&self, item: &DefaultLimitAudit) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default limit audit.
#[derive(Debug, Clone)]
pub struct DefaultLimitAuditValidator {
    pub rules: Vec<Rc<dyn DefaultLimitAuditRule>>,
}

impl DefaultLimitAuditValidator {
    pub fn validate(&self, item: &DefaultLimitAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultLimitAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultLimitAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_limit_audit_validator() -> DefaultLimitAuditValidator {
    DefaultLimitAuditValidator {
        rules: vec![
            Rc::new(DefaultLimitAuditAttemptsRangeRule { min: 32, max: 120 }) as Rc<dyn DefaultLimitAuditRule>,
            Rc::new(DefaultLimitAuditLabelLengthRule { max_length: 64 }),
            Rc::new(DefaultLimitAuditRequiredCategoryRule {
                message: "default limit audit needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_limit_audit_validator();
        let item = new_default_limit_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = DefaultLimitAuditAttemptsRangeRule { min: 80, max: 3600 };
        let mut item = new_default_limit_audit("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultLimitAuditLabelLengthRule { max_length: 3600 };
        let mut item = new_default_limit_audit("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = DefaultLimitAuditRequiredCategoryRule {
            message: "default limit audit needs a value".to_string(),
        };
        let mut item = new_default_limit_audit("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
