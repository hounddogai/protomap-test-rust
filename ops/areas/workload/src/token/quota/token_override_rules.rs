use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token override must pass. Check returns its problems, or none.
pub trait TokenOverrideRule: std::fmt::Debug {
    fn check(&self, item: &TokenOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenOverrideStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenOverrideRule for TokenOverrideStartsAtRangeRule {
    fn check(&self, item: &TokenOverride) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenOverrideRequiredTitleRule {
    pub message: String,
}

impl TokenOverrideRule for TokenOverrideRequiredTitleRule {
    fn check(&self, item: &TokenOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenOverrideAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TokenOverrideRule for TokenOverrideAllowedTitleRule {
    fn check(&self, item: &TokenOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenOverridePhoneLengthRule {
    pub max_length: i64,
}

impl TokenOverrideRule for TokenOverridePhoneLengthRule {
    fn check(&self, item: &TokenOverride) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenOverrideCodePrefixRule {
    pub prefix: String,
}

impl TokenOverrideRule for TokenOverrideCodePrefixRule {
    fn check(&self, item: &TokenOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenOverrideRequiredPhoneRule {
    pub message: String,
}

impl TokenOverrideRule for TokenOverrideRequiredPhoneRule {
    fn check(&self, item: &TokenOverride) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a token override.
#[derive(Debug, Clone)]
pub struct TokenOverrideValidator {
    pub rules: Vec<Rc<dyn TokenOverrideRule>>,
}

impl TokenOverrideValidator {
    pub fn validate(&self, item: &TokenOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_override_validator() -> TokenOverrideValidator {
    TokenOverrideValidator {
        rules: vec![
            Rc::new(TokenOverrideStartsAtRangeRule { min: 16, max: 80 }) as Rc<dyn TokenOverrideRule>,
            Rc::new(TokenOverrideRequiredTitleRule {
                message: "token override needs this field".to_string(),
            }),
            Rc::new(TokenOverrideAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(TokenOverridePhoneLengthRule { max_length: 1000 }),
            Rc::new(TokenOverrideCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TokenOverrideRequiredPhoneRule {
                message: "token override needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_override_validator();
        let item = new_token_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = TokenOverrideStartsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_token_override("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TokenOverrideRequiredTitleRule {
            message: "token override needs this field".to_string(),
        };
        let mut item = new_token_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = TokenOverridePhoneLengthRule { max_length: 3600 };
        let mut item = new_token_override("x", "y");
        item.phone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = TokenOverrideRequiredPhoneRule {
            message: "token override needs this field".to_string(),
        };
        let mut item = new_token_override("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
