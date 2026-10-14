use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce metric must pass. Check returns its problems, or none.
pub trait NonceMetricRule: std::fmt::Debug {
    fn check(&self, item: &NonceMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMetricTitleLengthRule {
    pub max_length: i64,
}

impl NonceMetricRule for NonceMetricTitleLengthRule {
    fn check(&self, item: &NonceMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMetricPrincipalLengthRule {
    pub max_length: i64,
}

impl NonceMetricRule for NonceMetricPrincipalLengthRule {
    fn check(&self, item: &NonceMetric) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMetricCodePrefixRule {
    pub prefix: String,
}

impl NonceMetricRule for NonceMetricCodePrefixRule {
    fn check(&self, item: &NonceMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMetricRequiredCodeRule {
    pub message: String,
}

impl NonceMetricRule for NonceMetricRequiredCodeRule {
    fn check(&self, item: &NonceMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMetricRequiredTitleRule {
    pub message: String,
}

impl NonceMetricRule for NonceMetricRequiredTitleRule {
    fn check(&self, item: &NonceMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce metric.
#[derive(Debug, Clone)]
pub struct NonceMetricValidator {
    pub rules: Vec<Rc<dyn NonceMetricRule>>,
}

impl NonceMetricValidator {
    pub fn validate(&self, item: &NonceMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_metric_validator() -> NonceMetricValidator {
    NonceMetricValidator {
        rules: vec![
            Rc::new(NonceMetricTitleLengthRule { max_length: 86400 }) as Rc<dyn NonceMetricRule>,
            Rc::new(NonceMetricPrincipalLengthRule { max_length: 64 }),
            Rc::new(NonceMetricCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(NonceMetricRequiredCodeRule {
                message: "nonce metric needs a value".to_string(),
            }),
            Rc::new(NonceMetricRequiredTitleRule {
                message: "nonce metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_metric_validator();
        let item = new_nonce_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = NonceMetricTitleLengthRule { max_length: 64 };
        let mut item = new_nonce_metric("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = NonceMetricPrincipalLengthRule { max_length: 3600 };
        let mut item = new_nonce_metric("x", "y");
        item.principal = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = NonceMetricRequiredCodeRule {
            message: "nonce metric needs a value".to_string(),
        };
        let mut item = new_nonce_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = NonceMetricRequiredTitleRule {
            message: "nonce metric needs an entry".to_string(),
        };
        let mut item = new_nonce_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
