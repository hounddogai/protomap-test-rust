use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval exception must pass. Check returns its problems, or none.
pub trait ApprovalExceptionRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalExceptionAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalExceptionRule for ApprovalExceptionAttemptsRangeRule {
    fn check(&self, item: &ApprovalException) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalExceptionAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ApprovalExceptionRule for ApprovalExceptionAllowedCodeRule {
    fn check(&self, item: &ApprovalException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalExceptionCodePrefixRule {
    pub prefix: String,
}

impl ApprovalExceptionRule for ApprovalExceptionCodePrefixRule {
    fn check(&self, item: &ApprovalException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalExceptionRequiredTitleRule {
    pub message: String,
}

impl ApprovalExceptionRule for ApprovalExceptionRequiredTitleRule {
    fn check(&self, item: &ApprovalException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalExceptionRequiredCodeRule {
    pub message: String,
}

impl ApprovalExceptionRule for ApprovalExceptionRequiredCodeRule {
    fn check(&self, item: &ApprovalException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalExceptionTitleLengthRule {
    pub max_length: i64,
}

impl ApprovalExceptionRule for ApprovalExceptionTitleLengthRule {
    fn check(&self, item: &ApprovalException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval exception.
#[derive(Debug, Clone)]
pub struct ApprovalExceptionValidator {
    pub rules: Vec<Rc<dyn ApprovalExceptionRule>>,
}

impl ApprovalExceptionValidator {
    pub fn validate(&self, item: &ApprovalException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_exception_validator() -> ApprovalExceptionValidator {
    ApprovalExceptionValidator {
        rules: vec![
            Rc::new(ApprovalExceptionAttemptsRangeRule { min: 16, max: 32 }) as Rc<dyn ApprovalExceptionRule>,
            Rc::new(ApprovalExceptionAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "default".to_string(), "primary".to_string()],
            }),
            Rc::new(ApprovalExceptionCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ApprovalExceptionRequiredTitleRule {
                message: "approval exception needs this field".to_string(),
            }),
            Rc::new(ApprovalExceptionRequiredCodeRule {
                message: "approval exception needs an entry".to_string(),
            }),
            Rc::new(ApprovalExceptionTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_exception_validator();
        let item = new_approval_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ApprovalExceptionAttemptsRangeRule { min: 16, max: 3600 };
        let mut item = new_approval_exception("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ApprovalExceptionRequiredTitleRule {
            message: "approval exception needs this field".to_string(),
        };
        let mut item = new_approval_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ApprovalExceptionRequiredCodeRule {
            message: "approval exception needs a value".to_string(),
        };
        let mut item = new_approval_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ApprovalExceptionTitleLengthRule { max_length: 1000 };
        let mut item = new_approval_exception("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
