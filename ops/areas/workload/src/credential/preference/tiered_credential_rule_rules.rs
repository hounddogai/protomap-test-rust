use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered credential rule must pass. Check returns its problems, or none.
pub trait TieredCredentialRuleRule: std::fmt::Debug {
    fn check(&self, item: &TieredCredentialRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRuleTitleLengthRule {
    pub max_length: i64,
}

impl TieredCredentialRuleRule for TieredCredentialRuleTitleLengthRule {
    fn check(&self, item: &TieredCredentialRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRuleRequiredTitleRule {
    pub message: String,
}

impl TieredCredentialRuleRule for TieredCredentialRuleRequiredTitleRule {
    fn check(&self, item: &TieredCredentialRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRuleCodeLengthRule {
    pub max_length: i64,
}

impl TieredCredentialRuleRule for TieredCredentialRuleCodeLengthRule {
    fn check(&self, item: &TieredCredentialRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRuleQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredCredentialRuleRule for TieredCredentialRuleQuantityRangeRule {
    fn check(&self, item: &TieredCredentialRule) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRuleExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredCredentialRuleRule for TieredCredentialRuleExpiresAtRangeRule {
    fn check(&self, item: &TieredCredentialRule) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRuleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredCredentialRuleRule for TieredCredentialRuleDiscountRangeRule {
    fn check(&self, item: &TieredCredentialRule) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered credential rule.
#[derive(Debug, Clone)]
pub struct TieredCredentialRuleValidator {
    pub rules: Vec<Rc<dyn TieredCredentialRuleRule>>,
}

impl TieredCredentialRuleValidator {
    pub fn validate(&self, item: &TieredCredentialRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredCredentialRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredCredentialRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_credential_rule_validator() -> TieredCredentialRuleValidator {
    TieredCredentialRuleValidator {
        rules: vec![
            Rc::new(TieredCredentialRuleTitleLengthRule { max_length: 3600 }) as Rc<dyn TieredCredentialRuleRule>,
            Rc::new(TieredCredentialRuleRequiredTitleRule {
                message: "tiered credential rule needs this field".to_string(),
            }),
            Rc::new(TieredCredentialRuleCodeLengthRule { max_length: 255 }),
            Rc::new(TieredCredentialRuleQuantityRangeRule { min: 1000, max: 1000 }),
            Rc::new(TieredCredentialRuleExpiresAtRangeRule { min: 255, max: 1000 }),
            Rc::new(TieredCredentialRuleDiscountRangeRule { min: 4.5, max: 3512.8 }),
        ],
    }
}
