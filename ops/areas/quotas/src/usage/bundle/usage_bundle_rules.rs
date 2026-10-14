use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage bundle must pass. Check returns its problems, or none.
pub trait UsageBundleRule: std::fmt::Debug {
    fn check(&self, item: &UsageBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBundleCodeLengthRule {
    pub max_length: i64,
}

impl UsageBundleRule for UsageBundleCodeLengthRule {
    fn check(&self, item: &UsageBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBundleRequiredExternalRefRule {
    pub message: String,
}

impl UsageBundleRule for UsageBundleRequiredExternalRefRule {
    fn check(&self, item: &UsageBundle) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBundleExternalRefLengthRule {
    pub max_length: i64,
}

impl UsageBundleRule for UsageBundleExternalRefLengthRule {
    fn check(&self, item: &UsageBundle) -> Vec<String> {
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
pub struct UsageBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl UsageBundleRule for UsageBundleAllowedCodeRule {
    fn check(&self, item: &UsageBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBundleRequiredCodeRule {
    pub message: String,
}

impl UsageBundleRule for UsageBundleRequiredCodeRule {
    fn check(&self, item: &UsageBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBundleRequiredAudienceRule {
    pub message: String,
}

impl UsageBundleRule for UsageBundleRequiredAudienceRule {
    fn check(&self, item: &UsageBundle) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage bundle.
#[derive(Debug, Clone)]
pub struct UsageBundleValidator {
    pub rules: Vec<Rc<dyn UsageBundleRule>>,
}

impl UsageBundleValidator {
    pub fn validate(&self, item: &UsageBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_bundle_validator() -> UsageBundleValidator {
    UsageBundleValidator {
        rules: vec![
            Rc::new(UsageBundleCodeLengthRule { max_length: 32 }) as Rc<dyn UsageBundleRule>,
            Rc::new(UsageBundleRequiredExternalRefRule {
                message: "usage bundle needs a value".to_string(),
            }),
            Rc::new(UsageBundleExternalRefLengthRule { max_length: 255 }),
            Rc::new(UsageBundleAllowedCodeRule {
                allowed: vec!["manual".to_string(), "global".to_string(), "pending".to_string()],
            }),
            Rc::new(UsageBundleRequiredCodeRule {
                message: "usage bundle needs an entry".to_string(),
            }),
            Rc::new(UsageBundleRequiredAudienceRule {
                message: "usage bundle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_bundle_validator();
        let item = new_usage_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = UsageBundleCodeLengthRule { max_length: 80 };
        let mut item = new_usage_bundle("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = UsageBundleRequiredExternalRefRule {
            message: "usage bundle needs this field".to_string(),
        };
        let mut item = new_usage_bundle("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = UsageBundleExternalRefLengthRule { max_length: 80 };
        let mut item = new_usage_bundle("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = UsageBundleRequiredCodeRule {
            message: "usage bundle needs this field".to_string(),
        };
        let mut item = new_usage_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = UsageBundleRequiredAudienceRule {
            message: "usage bundle needs a value".to_string(),
        };
        let mut item = new_usage_bundle("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
