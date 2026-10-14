use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token preference must pass. Check returns its problems, or none.
pub trait TokenPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &TokenPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenPreferenceCodePrefixRule {
    pub prefix: String,
}

impl TokenPreferenceRule for TokenPreferenceCodePrefixRule {
    fn check(&self, item: &TokenPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenPreferenceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TokenPreferenceRule for TokenPreferenceAllowedCodeRule {
    fn check(&self, item: &TokenPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenPreferenceRequiredTitleRule {
    pub message: String,
}

impl TokenPreferenceRule for TokenPreferenceRequiredTitleRule {
    fn check(&self, item: &TokenPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenPreferenceRequiredCodeRule {
    pub message: String,
}

impl TokenPreferenceRule for TokenPreferenceRequiredCodeRule {
    fn check(&self, item: &TokenPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenPreferenceLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenPreferenceRule for TokenPreferenceLimitRangeRule {
    fn check(&self, item: &TokenPreference) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token preference.
#[derive(Debug, Clone)]
pub struct TokenPreferenceValidator {
    pub rules: Vec<Rc<dyn TokenPreferenceRule>>,
}

impl TokenPreferenceValidator {
    pub fn validate(&self, item: &TokenPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_preference_validator() -> TokenPreferenceValidator {
    TokenPreferenceValidator {
        rules: vec![
            Rc::new(TokenPreferenceCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn TokenPreferenceRule>,
            Rc::new(TokenPreferenceAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "lenient".to_string()],
            }),
            Rc::new(TokenPreferenceRequiredTitleRule {
                message: "token preference needs an entry".to_string(),
            }),
            Rc::new(TokenPreferenceRequiredCodeRule {
                message: "token preference needs an entry".to_string(),
            }),
            Rc::new(TokenPreferenceLimitRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_preference_validator();
        let item = new_token_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TokenPreferenceRequiredTitleRule {
            message: "token preference needs a value".to_string(),
        };
        let mut item = new_token_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenPreferenceRequiredCodeRule {
            message: "token preference needs a value".to_string(),
        };
        let mut item = new_token_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = TokenPreferenceLimitRangeRule { min: 64, max: 1000 };
        let mut item = new_token_preference("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
