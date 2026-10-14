use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient canary tier must pass. Check returns its problems, or none.
pub trait LenientCanaryTierRule: std::fmt::Debug {
    fn check(&self, item: &LenientCanaryTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryTierSummaryLengthRule {
    pub max_length: i64,
}

impl LenientCanaryTierRule for LenientCanaryTierSummaryLengthRule {
    fn check(&self, item: &LenientCanaryTier) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryTierKeyIdLengthRule {
    pub max_length: i64,
}

impl LenientCanaryTierRule for LenientCanaryTierKeyIdLengthRule {
    fn check(&self, item: &LenientCanaryTier) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryTierRequiredSummaryRule {
    pub message: String,
}

impl LenientCanaryTierRule for LenientCanaryTierRequiredSummaryRule {
    fn check(&self, item: &LenientCanaryTier) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientCanaryTierRule for LenientCanaryTierAllowedCodeRule {
    fn check(&self, item: &LenientCanaryTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryTierCodePrefixRule {
    pub prefix: String,
}

impl LenientCanaryTierRule for LenientCanaryTierCodePrefixRule {
    fn check(&self, item: &LenientCanaryTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient canary tier.
#[derive(Debug, Clone)]
pub struct LenientCanaryTierValidator {
    pub rules: Vec<Rc<dyn LenientCanaryTierRule>>,
}

impl LenientCanaryTierValidator {
    pub fn validate(&self, item: &LenientCanaryTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientCanaryTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientCanaryTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_canary_tier_validator() -> LenientCanaryTierValidator {
    LenientCanaryTierValidator {
        rules: vec![
            Rc::new(LenientCanaryTierSummaryLengthRule { max_length: 255 }) as Rc<dyn LenientCanaryTierRule>,
            Rc::new(LenientCanaryTierKeyIdLengthRule { max_length: 120 }),
            Rc::new(LenientCanaryTierRequiredSummaryRule {
                message: "lenient canary tier needs an entry".to_string(),
            }),
            Rc::new(LenientCanaryTierAllowedCodeRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(LenientCanaryTierCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
