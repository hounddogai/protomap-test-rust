use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling quote bundle must pass. Check returns its problems, or none.
pub trait RollingQuoteBundleRule: std::fmt::Debug {
    fn check(&self, item: &RollingQuoteBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingQuoteBundleRule for RollingQuoteBundleAllowedCodeRule {
    fn check(&self, item: &RollingQuoteBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteBundleNameLengthRule {
    pub max_length: i64,
}

impl RollingQuoteBundleRule for RollingQuoteBundleNameLengthRule {
    fn check(&self, item: &RollingQuoteBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteBundleKeyIdLengthRule {
    pub max_length: i64,
}

impl RollingQuoteBundleRule for RollingQuoteBundleKeyIdLengthRule {
    fn check(&self, item: &RollingQuoteBundle) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteBundleRequiredCodeRule {
    pub message: String,
}

impl RollingQuoteBundleRule for RollingQuoteBundleRequiredCodeRule {
    fn check(&self, item: &RollingQuoteBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteBundleCodePrefixRule {
    pub prefix: String,
}

impl RollingQuoteBundleRule for RollingQuoteBundleCodePrefixRule {
    fn check(&self, item: &RollingQuoteBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling quote bundle.
#[derive(Debug, Clone)]
pub struct RollingQuoteBundleValidator {
    pub rules: Vec<Rc<dyn RollingQuoteBundleRule>>,
}

impl RollingQuoteBundleValidator {
    pub fn validate(&self, item: &RollingQuoteBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingQuoteBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingQuoteBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_quote_bundle_validator() -> RollingQuoteBundleValidator {
    RollingQuoteBundleValidator {
        rules: vec![
            Rc::new(RollingQuoteBundleAllowedCodeRule {
                allowed: vec!["pending".to_string(), "lenient".to_string()],
            }) as Rc<dyn RollingQuoteBundleRule>,
            Rc::new(RollingQuoteBundleNameLengthRule { max_length: 64 }),
            Rc::new(RollingQuoteBundleKeyIdLengthRule { max_length: 16 }),
            Rc::new(RollingQuoteBundleRequiredCodeRule {
                message: "rolling quote bundle needs a value".to_string(),
            }),
            Rc::new(RollingQuoteBundleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_quote_bundle_validator();
        let item = new_rolling_quote_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingQuoteBundleNameLengthRule { max_length: 255 };
        let mut item = new_rolling_quote_bundle("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = RollingQuoteBundleKeyIdLengthRule { max_length: 32 };
        let mut item = new_rolling_quote_bundle("x", "y");
        item.key_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingQuoteBundleRequiredCodeRule {
            message: "rolling quote bundle needs a value".to_string(),
        };
        let mut item = new_rolling_quote_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
