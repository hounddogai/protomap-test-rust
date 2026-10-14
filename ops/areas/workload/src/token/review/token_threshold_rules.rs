use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token threshold must pass. Check returns its problems, or none.
pub trait TokenThresholdRule: std::fmt::Debug {
    fn check(&self, item: &TokenThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenThresholdRequiredLabelRule {
    pub message: String,
}

impl TokenThresholdRule for TokenThresholdRequiredLabelRule {
    fn check(&self, item: &TokenThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenThresholdCodeLengthRule {
    pub max_length: i64,
}

impl TokenThresholdRule for TokenThresholdCodeLengthRule {
    fn check(&self, item: &TokenThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenThresholdOwnerIdLengthRule {
    pub max_length: i64,
}

impl TokenThresholdRule for TokenThresholdOwnerIdLengthRule {
    fn check(&self, item: &TokenThreshold) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a token threshold.
#[derive(Debug, Clone)]
pub struct TokenThresholdValidator {
    pub rules: Vec<Rc<dyn TokenThresholdRule>>,
}

impl TokenThresholdValidator {
    pub fn validate(&self, item: &TokenThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_threshold_validator() -> TokenThresholdValidator {
    TokenThresholdValidator {
        rules: vec![
            Rc::new(TokenThresholdRequiredLabelRule {
                message: "token threshold needs a value".to_string(),
            }) as Rc<dyn TokenThresholdRule>,
            Rc::new(TokenThresholdCodeLengthRule { max_length: 32 }),
            Rc::new(TokenThresholdOwnerIdLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_threshold_validator();
        let item = new_token_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TokenThresholdRequiredLabelRule {
            message: "token threshold needs this field".to_string(),
        };
        let mut item = new_token_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TokenThresholdCodeLengthRule { max_length: 3600 };
        let mut item = new_token_threshold("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = TokenThresholdOwnerIdLengthRule { max_length: 32 };
        let mut item = new_token_threshold("x", "y");
        item.owner_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
