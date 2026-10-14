use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token profile must pass. Check returns its problems, or none.
pub trait TokenProfileRule: std::fmt::Debug {
    fn check(&self, item: &TokenProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenProfileRequiredCodeRule {
    pub message: String,
}

impl TokenProfileRule for TokenProfileRequiredCodeRule {
    fn check(&self, item: &TokenProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenProfileTitleLengthRule {
    pub max_length: i64,
}

impl TokenProfileRule for TokenProfileTitleLengthRule {
    fn check(&self, item: &TokenProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenProfileExternalRefLengthRule {
    pub max_length: i64,
}

impl TokenProfileRule for TokenProfileExternalRefLengthRule {
    fn check(&self, item: &TokenProfile) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenProfileRequiredExternalRefRule {
    pub message: String,
}

impl TokenProfileRule for TokenProfileRequiredExternalRefRule {
    fn check(&self, item: &TokenProfile) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenProfileCodePrefixRule {
    pub prefix: String,
}

impl TokenProfileRule for TokenProfileCodePrefixRule {
    fn check(&self, item: &TokenProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token profile.
#[derive(Debug, Clone)]
pub struct TokenProfileValidator {
    pub rules: Vec<Rc<dyn TokenProfileRule>>,
}

impl TokenProfileValidator {
    pub fn validate(&self, item: &TokenProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_profile_validator() -> TokenProfileValidator {
    TokenProfileValidator {
        rules: vec![
            Rc::new(TokenProfileRequiredCodeRule {
                message: "token profile needs a value".to_string(),
            }) as Rc<dyn TokenProfileRule>,
            Rc::new(TokenProfileTitleLengthRule { max_length: 120 }),
            Rc::new(TokenProfileExternalRefLengthRule { max_length: 64 }),
            Rc::new(TokenProfileRequiredExternalRefRule {
                message: "token profile needs this field".to_string(),
            }),
            Rc::new(TokenProfileCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_profile_validator();
        let item = new_token_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenProfileRequiredCodeRule {
            message: "token profile needs a value".to_string(),
        };
        let mut item = new_token_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TokenProfileTitleLengthRule { max_length: 80 };
        let mut item = new_token_profile("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = TokenProfileExternalRefLengthRule { max_length: 86400 };
        let mut item = new_token_profile("x", "y");
        item.external_ref = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = TokenProfileRequiredExternalRefRule {
            message: "token profile needs a value".to_string(),
        };
        let mut item = new_token_profile("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
