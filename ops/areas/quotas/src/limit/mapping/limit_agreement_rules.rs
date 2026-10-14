use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a limit agreement must pass. Check returns its problems, or none.
pub trait LimitAgreementRule: std::fmt::Debug {
    fn check(&self, item: &LimitAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAgreementCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitAgreementRule for LimitAgreementCostRangeRule {
    fn check(&self, item: &LimitAgreement) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAgreementRequiredCodeRule {
    pub message: String,
}

impl LimitAgreementRule for LimitAgreementRequiredCodeRule {
    fn check(&self, item: &LimitAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAgreementAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LimitAgreementRule for LimitAgreementAllowedTitleRule {
    fn check(&self, item: &LimitAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is above maximum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAgreementCodePrefixRule {
    pub prefix: String,
}

impl LimitAgreementRule for LimitAgreementCodePrefixRule {
    fn check(&self, item: &LimitAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAgreementRequiredTitleRule {
    pub message: String,
}

impl LimitAgreementRule for LimitAgreementRequiredTitleRule {
    fn check(&self, item: &LimitAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAgreementRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitAgreementRule for LimitAgreementRatioRangeRule {
    fn check(&self, item: &LimitAgreement) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit agreement.
#[derive(Debug, Clone)]
pub struct LimitAgreementValidator {
    pub rules: Vec<Rc<dyn LimitAgreementRule>>,
}

impl LimitAgreementValidator {
    pub fn validate(&self, item: &LimitAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_agreement_validator() -> LimitAgreementValidator {
    LimitAgreementValidator {
        rules: vec![
            Rc::new(LimitAgreementCostRangeRule { min: 8.0, max: 3182.1 }) as Rc<dyn LimitAgreementRule>,
            Rc::new(LimitAgreementRequiredCodeRule {
                message: "limit agreement needs a value".to_string(),
            }),
            Rc::new(LimitAgreementAllowedTitleRule {
                allowed: vec!["weighted".to_string(), "tiered".to_string()],
            }),
            Rc::new(LimitAgreementCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LimitAgreementRequiredTitleRule {
                message: "limit agreement needs an entry".to_string(),
            }),
            Rc::new(LimitAgreementRatioRangeRule { min: 9.1, max: 2476.2 }),
        ],
    }
}
