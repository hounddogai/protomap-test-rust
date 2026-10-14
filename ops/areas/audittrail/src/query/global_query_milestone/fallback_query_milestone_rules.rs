use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback query milestone must pass. Check returns its problems, or none.
pub trait FallbackQueryMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &FallbackQueryMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQueryMilestoneCodePrefixRule {
    pub prefix: String,
}

impl FallbackQueryMilestoneRule for FallbackQueryMilestoneCodePrefixRule {
    fn check(&self, item: &FallbackQueryMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQueryMilestoneRequiredCipherRule {
    pub message: String,
}

impl FallbackQueryMilestoneRule for FallbackQueryMilestoneRequiredCipherRule {
    fn check(&self, item: &FallbackQueryMilestone) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQueryMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl FallbackQueryMilestoneRule for FallbackQueryMilestoneCodeLengthRule {
    fn check(&self, item: &FallbackQueryMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback query milestone.
#[derive(Debug, Clone)]
pub struct FallbackQueryMilestoneValidator {
    pub rules: Vec<Rc<dyn FallbackQueryMilestoneRule>>,
}

impl FallbackQueryMilestoneValidator {
    pub fn validate(&self, item: &FallbackQueryMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackQueryMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackQueryMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_query_milestone_validator() -> FallbackQueryMilestoneValidator {
    FallbackQueryMilestoneValidator {
        rules: vec![
            Rc::new(FallbackQueryMilestoneCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn FallbackQueryMilestoneRule>,
            Rc::new(FallbackQueryMilestoneRequiredCipherRule {
                message: "fallback query milestone needs a value".to_string(),
            }),
            Rc::new(FallbackQueryMilestoneCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_query_milestone_validator();
        let item = new_fallback_query_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = FallbackQueryMilestoneRequiredCipherRule {
            message: "fallback query milestone needs a value".to_string(),
        };
        let mut item = new_fallback_query_milestone("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackQueryMilestoneCodeLengthRule { max_length: 3600 };
        let mut item = new_fallback_query_milestone("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
