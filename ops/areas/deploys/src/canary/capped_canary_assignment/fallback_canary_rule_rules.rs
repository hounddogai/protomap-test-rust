use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback canary rule must pass. Check returns its problems, or none.
pub trait FallbackCanaryRuleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackCanaryRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCanaryRuleReviewerLengthRule {
    pub max_length: i64,
}

impl FallbackCanaryRuleRule for FallbackCanaryRuleReviewerLengthRule {
    fn check(&self, item: &FallbackCanaryRule) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCanaryRuleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackCanaryRuleRule for FallbackCanaryRuleEndsAtRangeRule {
    fn check(&self, item: &FallbackCanaryRule) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCanaryRuleRequiredCodeRule {
    pub message: String,
}

impl FallbackCanaryRuleRule for FallbackCanaryRuleRequiredCodeRule {
    fn check(&self, item: &FallbackCanaryRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCanaryRuleCodePrefixRule {
    pub prefix: String,
}

impl FallbackCanaryRuleRule for FallbackCanaryRuleCodePrefixRule {
    fn check(&self, item: &FallbackCanaryRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCanaryRuleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackCanaryRuleRule for FallbackCanaryRuleCeilingRangeRule {
    fn check(&self, item: &FallbackCanaryRule) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCanaryRuleLabelLengthRule {
    pub max_length: i64,
}

impl FallbackCanaryRuleRule for FallbackCanaryRuleLabelLengthRule {
    fn check(&self, item: &FallbackCanaryRule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback canary rule.
#[derive(Debug, Clone)]
pub struct FallbackCanaryRuleValidator {
    pub rules: Vec<Rc<dyn FallbackCanaryRuleRule>>,
}

impl FallbackCanaryRuleValidator {
    pub fn validate(&self, item: &FallbackCanaryRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackCanaryRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackCanaryRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_canary_rule_validator() -> FallbackCanaryRuleValidator {
    FallbackCanaryRuleValidator {
        rules: vec![
            Rc::new(FallbackCanaryRuleReviewerLengthRule { max_length: 32 }) as Rc<dyn FallbackCanaryRuleRule>,
            Rc::new(FallbackCanaryRuleEndsAtRangeRule { min: 120, max: 86400 }),
            Rc::new(FallbackCanaryRuleRequiredCodeRule {
                message: "fallback canary rule needs an entry".to_string(),
            }),
            Rc::new(FallbackCanaryRuleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(FallbackCanaryRuleCeilingRangeRule { min: 2.5, max: 3288.5 }),
            Rc::new(FallbackCanaryRuleLabelLengthRule { max_length: 255 }),
        ],
    }
}
