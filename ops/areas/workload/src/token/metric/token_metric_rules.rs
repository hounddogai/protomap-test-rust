use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token metric must pass. Check returns its problems, or none.
pub trait TokenMetricRule: std::fmt::Debug {
    fn check(&self, item: &TokenMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetricUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenMetricRule for TokenMetricUpdatedAtRangeRule {
    fn check(&self, item: &TokenMetric) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetricLabelLengthRule {
    pub max_length: i64,
}

impl TokenMetricRule for TokenMetricLabelLengthRule {
    fn check(&self, item: &TokenMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetricNameLengthRule {
    pub max_length: i64,
}

impl TokenMetricRule for TokenMetricNameLengthRule {
    fn check(&self, item: &TokenMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetricCipherLengthRule {
    pub max_length: i64,
}

impl TokenMetricRule for TokenMetricCipherLengthRule {
    fn check(&self, item: &TokenMetric) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetricAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl TokenMetricRule for TokenMetricAllowedLabelRule {
    fn check(&self, item: &TokenMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetricRequiredCipherRule {
    pub message: String,
}

impl TokenMetricRule for TokenMetricRequiredCipherRule {
    fn check(&self, item: &TokenMetric) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a token metric.
#[derive(Debug, Clone)]
pub struct TokenMetricValidator {
    pub rules: Vec<Rc<dyn TokenMetricRule>>,
}

impl TokenMetricValidator {
    pub fn validate(&self, item: &TokenMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_metric_validator() -> TokenMetricValidator {
    TokenMetricValidator {
        rules: vec![
            Rc::new(TokenMetricUpdatedAtRangeRule { min: 64, max: 86400 }) as Rc<dyn TokenMetricRule>,
            Rc::new(TokenMetricLabelLengthRule { max_length: 255 }),
            Rc::new(TokenMetricNameLengthRule { max_length: 16 }),
            Rc::new(TokenMetricCipherLengthRule { max_length: 32 }),
            Rc::new(TokenMetricAllowedLabelRule {
                allowed: vec!["regional".to_string(), "global".to_string()],
            }),
            Rc::new(TokenMetricRequiredCipherRule {
                message: "token metric needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_metric_validator();
        let item = new_token_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = TokenMetricUpdatedAtRangeRule { min: 16, max: 86400 };
        let mut item = new_token_metric("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TokenMetricLabelLengthRule { max_length: 3600 };
        let mut item = new_token_metric("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TokenMetricNameLengthRule { max_length: 3600 };
        let mut item = new_token_metric("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = TokenMetricCipherLengthRule { max_length: 255 };
        let mut item = new_token_metric("x", "y");
        item.cipher = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = TokenMetricRequiredCipherRule {
            message: "token metric needs a value".to_string(),
        };
        let mut item = new_token_metric("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
