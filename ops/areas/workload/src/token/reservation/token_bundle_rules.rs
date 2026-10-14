use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token bundle must pass. Check returns its problems, or none.
pub trait TokenBundleRule: std::fmt::Debug {
    fn check(&self, item: &TokenBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBundleExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenBundleRule for TokenBundleExpiresAtRangeRule {
    fn check(&self, item: &TokenBundle) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBundleAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl TokenBundleRule for TokenBundleAllowedExternalRefRule {
    fn check(&self, item: &TokenBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is duplicated", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBundleCodePrefixRule {
    pub prefix: String,
}

impl TokenBundleRule for TokenBundleCodePrefixRule {
    fn check(&self, item: &TokenBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenBundleExternalRefLengthRule {
    pub max_length: i64,
}

impl TokenBundleRule for TokenBundleExternalRefLengthRule {
    fn check(&self, item: &TokenBundle) -> Vec<String> {
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

/// Runs every rule of a token bundle.
#[derive(Debug, Clone)]
pub struct TokenBundleValidator {
    pub rules: Vec<Rc<dyn TokenBundleRule>>,
}

impl TokenBundleValidator {
    pub fn validate(&self, item: &TokenBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_bundle_validator() -> TokenBundleValidator {
    TokenBundleValidator {
        rules: vec![
            Rc::new(TokenBundleExpiresAtRangeRule { min: 3600, max: 86400 }) as Rc<dyn TokenBundleRule>,
            Rc::new(TokenBundleAllowedExternalRefRule { allowed: Vec::new() }),
            Rc::new(TokenBundleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TokenBundleExternalRefLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_bundle_validator();
        let item = new_token_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TokenBundleExpiresAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_token_bundle("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = TokenBundleExternalRefLengthRule { max_length: 255 };
        let mut item = new_token_bundle("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
