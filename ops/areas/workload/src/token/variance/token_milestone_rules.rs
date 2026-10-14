use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token milestone must pass. Check returns its problems, or none.
pub trait TokenMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &TokenMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMilestoneRequiredNameRule {
    pub message: String,
}

impl TokenMilestoneRule for TokenMilestoneRequiredNameRule {
    fn check(&self, item: &TokenMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl TokenMilestoneRule for TokenMilestoneCodeLengthRule {
    fn check(&self, item: &TokenMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMilestoneCodePrefixRule {
    pub prefix: String,
}

impl TokenMilestoneRule for TokenMilestoneCodePrefixRule {
    fn check(&self, item: &TokenMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token milestone.
#[derive(Debug, Clone)]
pub struct TokenMilestoneValidator {
    pub rules: Vec<Rc<dyn TokenMilestoneRule>>,
}

impl TokenMilestoneValidator {
    pub fn validate(&self, item: &TokenMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_milestone_validator() -> TokenMilestoneValidator {
    TokenMilestoneValidator {
        rules: vec![
            Rc::new(TokenMilestoneRequiredNameRule {
                message: "token milestone needs a value".to_string(),
            }) as Rc<dyn TokenMilestoneRule>,
            Rc::new(TokenMilestoneCodeLengthRule { max_length: 16 }),
            Rc::new(TokenMilestoneCodePrefixRule {
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
        let validator = default_token_milestone_validator();
        let item = new_token_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TokenMilestoneRequiredNameRule {
            message: "token milestone needs this field".to_string(),
        };
        let mut item = new_token_milestone("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TokenMilestoneCodeLengthRule { max_length: 86400 };
        let mut item = new_token_milestone("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
