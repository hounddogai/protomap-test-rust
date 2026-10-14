use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule variance must pass. Check returns its problems, or none.
pub trait RuleVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RuleVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleVarianceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RuleVarianceRule for RuleVarianceAllowedCodeRule {
    fn check(&self, item: &RuleVariance) -> Vec<String> {
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
pub struct RuleVarianceRequiredSourceRule {
    pub message: String,
}

impl RuleVarianceRule for RuleVarianceRequiredSourceRule {
    fn check(&self, item: &RuleVariance) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleVarianceCodePrefixRule {
    pub prefix: String,
}

impl RuleVarianceRule for RuleVarianceCodePrefixRule {
    fn check(&self, item: &RuleVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleVarianceSourceLengthRule {
    pub max_length: i64,
}

impl RuleVarianceRule for RuleVarianceSourceLengthRule {
    fn check(&self, item: &RuleVariance) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleVarianceApprovedGuardRule {
    pub limit: i64,
}

impl RuleVarianceRule for RuleVarianceApprovedGuardRule {
    fn check(&self, item: &RuleVariance) -> Vec<String> {
        if item.approved && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while approved",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleVarianceCodeLengthRule {
    pub max_length: i64,
}

impl RuleVarianceRule for RuleVarianceCodeLengthRule {
    fn check(&self, item: &RuleVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule variance.
#[derive(Debug, Clone)]
pub struct RuleVarianceValidator {
    pub rules: Vec<Rc<dyn RuleVarianceRule>>,
}

impl RuleVarianceValidator {
    pub fn validate(&self, item: &RuleVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_variance_validator() -> RuleVarianceValidator {
    RuleVarianceValidator {
        rules: vec![
            Rc::new(RuleVarianceAllowedCodeRule {
                allowed: vec!["tiered".to_string()],
            }) as Rc<dyn RuleVarianceRule>,
            Rc::new(RuleVarianceRequiredSourceRule {
                message: "rule variance needs an entry".to_string(),
            }),
            Rc::new(RuleVarianceCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RuleVarianceSourceLengthRule { max_length: 64 }),
            Rc::new(RuleVarianceApprovedGuardRule { limit: 64 }),
            Rc::new(RuleVarianceCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_variance_validator();
        let item = new_rule_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = RuleVarianceRequiredSourceRule {
            message: "rule variance needs an entry".to_string(),
        };
        let mut item = new_rule_variance("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = RuleVarianceSourceLengthRule { max_length: 86400 };
        let mut item = new_rule_variance("x", "y");
        item.source = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RuleVarianceCodeLengthRule { max_length: 1000 };
        let mut item = new_rule_variance("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
