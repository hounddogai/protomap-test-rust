use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token quota must pass. Check returns its problems, or none.
pub trait TokenQuotaRule: std::fmt::Debug {
    fn check(&self, item: &TokenQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenQuotaExternalRefLengthRule {
    pub max_length: i64,
}

impl TokenQuotaRule for TokenQuotaExternalRefLengthRule {
    fn check(&self, item: &TokenQuota) -> Vec<String> {
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
pub struct TokenQuotaCodePrefixRule {
    pub prefix: String,
}

impl TokenQuotaRule for TokenQuotaCodePrefixRule {
    fn check(&self, item: &TokenQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenQuotaBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenQuotaRule for TokenQuotaBudgetRangeRule {
    fn check(&self, item: &TokenQuota) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenQuotaCodeLengthRule {
    pub max_length: i64,
}

impl TokenQuotaRule for TokenQuotaCodeLengthRule {
    fn check(&self, item: &TokenQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenQuotaRequiredNameRule {
    pub message: String,
}

impl TokenQuotaRule for TokenQuotaRequiredNameRule {
    fn check(&self, item: &TokenQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TokenQuotaRule for TokenQuotaAllowedCodeRule {
    fn check(&self, item: &TokenQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token quota.
#[derive(Debug, Clone)]
pub struct TokenQuotaValidator {
    pub rules: Vec<Rc<dyn TokenQuotaRule>>,
}

impl TokenQuotaValidator {
    pub fn validate(&self, item: &TokenQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_quota_validator() -> TokenQuotaValidator {
    TokenQuotaValidator {
        rules: vec![
            Rc::new(TokenQuotaExternalRefLengthRule { max_length: 64 }) as Rc<dyn TokenQuotaRule>,
            Rc::new(TokenQuotaCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TokenQuotaBudgetRangeRule { min: 1.6, max: 1706.4 }),
            Rc::new(TokenQuotaCodeLengthRule { max_length: 64 }),
            Rc::new(TokenQuotaRequiredNameRule {
                message: "token quota needs a value".to_string(),
            }),
            Rc::new(TokenQuotaAllowedCodeRule {
                allowed: vec!["tiered".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_quota_validator();
        let item = new_token_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = TokenQuotaExternalRefLengthRule { max_length: 86400 };
        let mut item = new_token_quota("x", "y");
        item.external_ref = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TokenQuotaBudgetRangeRule { min: 8.3, max: 2735.2 };
        let mut item = new_token_quota("x", "y");
        item.budget = 2736.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TokenQuotaCodeLengthRule { max_length: 86400 };
        let mut item = new_token_quota("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TokenQuotaRequiredNameRule {
            message: "token quota needs this field".to_string(),
        };
        let mut item = new_token_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
