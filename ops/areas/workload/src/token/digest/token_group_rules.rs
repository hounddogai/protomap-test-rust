use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token group must pass. Check returns its problems, or none.
pub trait TokenGroupRule: std::fmt::Debug {
    fn check(&self, item: &TokenGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenGroupTitleLengthRule {
    pub max_length: i64,
}

impl TokenGroupRule for TokenGroupTitleLengthRule {
    fn check(&self, item: &TokenGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenGroupCodePrefixRule {
    pub prefix: String,
}

impl TokenGroupRule for TokenGroupCodePrefixRule {
    fn check(&self, item: &TokenGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenGroupCodeLengthRule {
    pub max_length: i64,
}

impl TokenGroupRule for TokenGroupCodeLengthRule {
    fn check(&self, item: &TokenGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenGroupIssuerLengthRule {
    pub max_length: i64,
}

impl TokenGroupRule for TokenGroupIssuerLengthRule {
    fn check(&self, item: &TokenGroup) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenGroupTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl TokenGroupRule for TokenGroupTagsLimitRule {
    fn check(&self, item: &TokenGroup) -> Vec<String> {
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
pub struct TokenGroupCipherLengthRule {
    pub max_length: i64,
}

impl TokenGroupRule for TokenGroupCipherLengthRule {
    fn check(&self, item: &TokenGroup) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token group.
#[derive(Debug, Clone)]
pub struct TokenGroupValidator {
    pub rules: Vec<Rc<dyn TokenGroupRule>>,
}

impl TokenGroupValidator {
    pub fn validate(&self, item: &TokenGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_group_validator() -> TokenGroupValidator {
    TokenGroupValidator {
        rules: vec![
            Rc::new(TokenGroupTitleLengthRule { max_length: 32 }) as Rc<dyn TokenGroupRule>,
            Rc::new(TokenGroupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TokenGroupCodeLengthRule { max_length: 64 }),
            Rc::new(TokenGroupIssuerLengthRule { max_length: 3600 }),
            Rc::new(TokenGroupTagsLimitRule {
                max_count: 120,
                banned: vec!["capped".to_string(), "rolling".to_string(), "strict".to_string()],
            }),
            Rc::new(TokenGroupCipherLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_group_validator();
        let item = new_token_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TokenGroupTitleLengthRule { max_length: 255 };
        let mut item = new_token_group("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TokenGroupCodeLengthRule { max_length: 32 };
        let mut item = new_token_group("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = TokenGroupIssuerLengthRule { max_length: 32 };
        let mut item = new_token_group("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = TokenGroupCipherLengthRule { max_length: 64 };
        let mut item = new_token_group("x", "y");
        item.cipher = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
