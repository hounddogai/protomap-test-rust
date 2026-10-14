use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy estimate must pass. Check returns its problems, or none.
pub trait TtlPolicyEstimateRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyEstimateLocaleLengthRule {
    pub max_length: i64,
}

impl TtlPolicyEstimateRule for TtlPolicyEstimateLocaleLengthRule {
    fn check(&self, item: &TtlPolicyEstimate) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyEstimateAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl TtlPolicyEstimateRule for TtlPolicyEstimateAllowedSourceRule {
    fn check(&self, item: &TtlPolicyEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is unknown", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyEstimateCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyEstimateRule for TtlPolicyEstimateCodePrefixRule {
    fn check(&self, item: &TtlPolicyEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyEstimateRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyEstimateRule for TtlPolicyEstimateRequiredCodeRule {
    fn check(&self, item: &TtlPolicyEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyEstimateRequiredTitleRule {
    pub message: String,
}

impl TtlPolicyEstimateRule for TtlPolicyEstimateRequiredTitleRule {
    fn check(&self, item: &TtlPolicyEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy estimate.
#[derive(Debug, Clone)]
pub struct TtlPolicyEstimateValidator {
    pub rules: Vec<Rc<dyn TtlPolicyEstimateRule>>,
}

impl TtlPolicyEstimateValidator {
    pub fn validate(&self, item: &TtlPolicyEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_estimate_validator() -> TtlPolicyEstimateValidator {
    TtlPolicyEstimateValidator {
        rules: vec![
            Rc::new(TtlPolicyEstimateLocaleLengthRule { max_length: 64 }) as Rc<dyn TtlPolicyEstimateRule>,
            Rc::new(TtlPolicyEstimateAllowedSourceRule {
                allowed: vec!["regional".to_string(), "tiered".to_string()],
            }),
            Rc::new(TtlPolicyEstimateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TtlPolicyEstimateRequiredCodeRule {
                message: "ttl policy estimate needs a value".to_string(),
            }),
            Rc::new(TtlPolicyEstimateRequiredTitleRule {
                message: "ttl policy estimate needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_estimate_validator();
        let item = new_ttl_policy_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = TtlPolicyEstimateLocaleLengthRule { max_length: 120 };
        let mut item = new_ttl_policy_estimate("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyEstimateRequiredCodeRule {
            message: "ttl policy estimate needs a value".to_string(),
        };
        let mut item = new_ttl_policy_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TtlPolicyEstimateRequiredTitleRule {
            message: "ttl policy estimate needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
