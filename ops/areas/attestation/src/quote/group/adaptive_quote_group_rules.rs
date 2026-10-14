use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive quote group must pass. Check returns its problems, or none.
pub trait AdaptiveQuoteGroupRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveQuoteGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteGroupCountryLengthRule {
    pub max_length: i64,
}

impl AdaptiveQuoteGroupRule for AdaptiveQuoteGroupCountryLengthRule {
    fn check(&self, item: &AdaptiveQuoteGroup) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteGroupLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveQuoteGroupRule for AdaptiveQuoteGroupLabelLengthRule {
    fn check(&self, item: &AdaptiveQuoteGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteGroupRequiredCodeRule {
    pub message: String,
}

impl AdaptiveQuoteGroupRule for AdaptiveQuoteGroupRequiredCodeRule {
    fn check(&self, item: &AdaptiveQuoteGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteGroupThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveQuoteGroupRule for AdaptiveQuoteGroupThresholdRangeRule {
    fn check(&self, item: &AdaptiveQuoteGroup) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteGroupRequiredCountryRule {
    pub message: String,
}

impl AdaptiveQuoteGroupRule for AdaptiveQuoteGroupRequiredCountryRule {
    fn check(&self, item: &AdaptiveQuoteGroup) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteGroupCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveQuoteGroupRule for AdaptiveQuoteGroupCodePrefixRule {
    fn check(&self, item: &AdaptiveQuoteGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive quote group.
#[derive(Debug, Clone)]
pub struct AdaptiveQuoteGroupValidator {
    pub rules: Vec<Rc<dyn AdaptiveQuoteGroupRule>>,
}

impl AdaptiveQuoteGroupValidator {
    pub fn validate(&self, item: &AdaptiveQuoteGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveQuoteGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveQuoteGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_quote_group_validator() -> AdaptiveQuoteGroupValidator {
    AdaptiveQuoteGroupValidator {
        rules: vec![
            Rc::new(AdaptiveQuoteGroupCountryLengthRule { max_length: 1000 }) as Rc<dyn AdaptiveQuoteGroupRule>,
            Rc::new(AdaptiveQuoteGroupLabelLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveQuoteGroupRequiredCodeRule {
                message: "adaptive quote group needs a value".to_string(),
            }),
            Rc::new(AdaptiveQuoteGroupThresholdRangeRule { min: 0.2, max: 1059.5 }),
            Rc::new(AdaptiveQuoteGroupRequiredCountryRule {
                message: "adaptive quote group needs an entry".to_string(),
            }),
            Rc::new(AdaptiveQuoteGroupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
