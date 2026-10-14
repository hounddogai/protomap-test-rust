use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient nonce preference must pass. Check returns its problems, or none.
pub trait LenientNoncePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LenientNoncePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNoncePreferenceCodeLengthRule {
    pub max_length: i64,
}

impl LenientNoncePreferenceRule for LenientNoncePreferenceCodeLengthRule {
    fn check(&self, item: &LenientNoncePreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNoncePreferenceSourceLengthRule {
    pub max_length: i64,
}

impl LenientNoncePreferenceRule for LenientNoncePreferenceSourceLengthRule {
    fn check(&self, item: &LenientNoncePreference) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNoncePreferenceRequiredTitleRule {
    pub message: String,
}

impl LenientNoncePreferenceRule for LenientNoncePreferenceRequiredTitleRule {
    fn check(&self, item: &LenientNoncePreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNoncePreferenceDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientNoncePreferenceRule for LenientNoncePreferenceDiscountRangeRule {
    fn check(&self, item: &LenientNoncePreference) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNoncePreferenceCodePrefixRule {
    pub prefix: String,
}

impl LenientNoncePreferenceRule for LenientNoncePreferenceCodePrefixRule {
    fn check(&self, item: &LenientNoncePreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNoncePreferenceRequiredCodeRule {
    pub message: String,
}

impl LenientNoncePreferenceRule for LenientNoncePreferenceRequiredCodeRule {
    fn check(&self, item: &LenientNoncePreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient nonce preference.
#[derive(Debug, Clone)]
pub struct LenientNoncePreferenceValidator {
    pub rules: Vec<Rc<dyn LenientNoncePreferenceRule>>,
}

impl LenientNoncePreferenceValidator {
    pub fn validate(&self, item: &LenientNoncePreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientNoncePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientNoncePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_nonce_preference_validator() -> LenientNoncePreferenceValidator {
    LenientNoncePreferenceValidator {
        rules: vec![
            Rc::new(LenientNoncePreferenceCodeLengthRule { max_length: 16 }) as Rc<dyn LenientNoncePreferenceRule>,
            Rc::new(LenientNoncePreferenceSourceLengthRule { max_length: 64 }),
            Rc::new(LenientNoncePreferenceRequiredTitleRule {
                message: "lenient nonce preference needs a value".to_string(),
            }),
            Rc::new(LenientNoncePreferenceDiscountRangeRule { min: 2.8, max: 4768.4 }),
            Rc::new(LenientNoncePreferenceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LenientNoncePreferenceRequiredCodeRule {
                message: "lenient nonce preference needs a value".to_string(),
            }),
        ],
    }
}
