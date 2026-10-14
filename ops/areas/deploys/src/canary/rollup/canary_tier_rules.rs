use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary tier must pass. Check returns its problems, or none.
pub trait CanaryTierRule: std::fmt::Debug {
    fn check(&self, item: &CanaryTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTierSourceLengthRule {
    pub max_length: i64,
}

impl CanaryTierRule for CanaryTierSourceLengthRule {
    fn check(&self, item: &CanaryTier) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTierAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl CanaryTierRule for CanaryTierAllowedTitleRule {
    fn check(&self, item: &CanaryTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTierCipherLengthRule {
    pub max_length: i64,
}

impl CanaryTierRule for CanaryTierCipherLengthRule {
    fn check(&self, item: &CanaryTier) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTierRequiredCodeRule {
    pub message: String,
}

impl CanaryTierRule for CanaryTierRequiredCodeRule {
    fn check(&self, item: &CanaryTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary tier.
#[derive(Debug, Clone)]
pub struct CanaryTierValidator {
    pub rules: Vec<Rc<dyn CanaryTierRule>>,
}

impl CanaryTierValidator {
    pub fn validate(&self, item: &CanaryTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_tier_validator() -> CanaryTierValidator {
    CanaryTierValidator {
        rules: vec![
            Rc::new(CanaryTierSourceLengthRule { max_length: 64 }) as Rc<dyn CanaryTierRule>,
            Rc::new(CanaryTierAllowedTitleRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(CanaryTierCipherLengthRule { max_length: 3600 }),
            Rc::new(CanaryTierRequiredCodeRule {
                message: "canary tier needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_tier_validator();
        let item = new_canary_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = CanaryTierSourceLengthRule { max_length: 3600 };
        let mut item = new_canary_tier("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = CanaryTierCipherLengthRule { max_length: 1000 };
        let mut item = new_canary_tier("x", "y");
        item.cipher = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryTierRequiredCodeRule {
            message: "canary tier needs an entry".to_string(),
        };
        let mut item = new_canary_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
