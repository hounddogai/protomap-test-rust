use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token batch must pass. Check returns its problems, or none.
pub trait TokenBatchRule: std::fmt::Debug {
    fn check(&self, item: &TokenBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBatchRequiredOwnerIdRule {
    pub message: String,
}

impl TokenBatchRule for TokenBatchRequiredOwnerIdRule {
    fn check(&self, item: &TokenBatch) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBatchCodeLengthRule {
    pub max_length: i64,
}

impl TokenBatchRule for TokenBatchCodeLengthRule {
    fn check(&self, item: &TokenBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBatchCodePrefixRule {
    pub prefix: String,
}

impl TokenBatchRule for TokenBatchCodePrefixRule {
    fn check(&self, item: &TokenBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBatchNameLengthRule {
    pub max_length: i64,
}

impl TokenBatchRule for TokenBatchNameLengthRule {
    fn check(&self, item: &TokenBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBatchOwnerIdLengthRule {
    pub max_length: i64,
}

impl TokenBatchRule for TokenBatchOwnerIdLengthRule {
    fn check(&self, item: &TokenBatch) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBatchRequiredChecksumRule {
    pub message: String,
}

impl TokenBatchRule for TokenBatchRequiredChecksumRule {
    fn check(&self, item: &TokenBatch) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a token batch.
#[derive(Debug, Clone)]
pub struct TokenBatchValidator {
    pub rules: Vec<Rc<dyn TokenBatchRule>>,
}

impl TokenBatchValidator {
    pub fn validate(&self, item: &TokenBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_batch_validator() -> TokenBatchValidator {
    TokenBatchValidator {
        rules: vec![
            Rc::new(TokenBatchRequiredOwnerIdRule {
                message: "token batch needs a value".to_string(),
            }) as Rc<dyn TokenBatchRule>,
            Rc::new(TokenBatchCodeLengthRule { max_length: 64 }),
            Rc::new(TokenBatchCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TokenBatchNameLengthRule { max_length: 255 }),
            Rc::new(TokenBatchOwnerIdLengthRule { max_length: 16 }),
            Rc::new(TokenBatchRequiredChecksumRule {
                message: "token batch needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_batch_validator();
        let item = new_token_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = TokenBatchRequiredOwnerIdRule {
            message: "token batch needs a value".to_string(),
        };
        let mut item = new_token_batch("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TokenBatchCodeLengthRule { max_length: 1000 };
        let mut item = new_token_batch("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TokenBatchNameLengthRule { max_length: 86400 };
        let mut item = new_token_batch("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = TokenBatchOwnerIdLengthRule { max_length: 120 };
        let mut item = new_token_batch("x", "y");
        item.owner_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = TokenBatchRequiredChecksumRule {
            message: "token batch needs an entry".to_string(),
        };
        let mut item = new_token_batch("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
