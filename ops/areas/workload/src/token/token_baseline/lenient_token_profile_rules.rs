use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient token profile must pass. Check returns its problems, or none.
pub trait LenientTokenProfileRule: std::fmt::Debug {
    fn check(&self, item: &LenientTokenProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenProfileRequiredCodeRule {
    pub message: String,
}

impl LenientTokenProfileRule for LenientTokenProfileRequiredCodeRule {
    fn check(&self, item: &LenientTokenProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenProfileActiveGuardRule {
    pub limit: i64,
}

impl LenientTokenProfileRule for LenientTokenProfileActiveGuardRule {
    fn check(&self, item: &LenientTokenProfile) -> Vec<String> {
        if item.active && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while active",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenProfileNameLengthRule {
    pub max_length: i64,
}

impl LenientTokenProfileRule for LenientTokenProfileNameLengthRule {
    fn check(&self, item: &LenientTokenProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenProfileRequiredScopeRule {
    pub message: String,
}

impl LenientTokenProfileRule for LenientTokenProfileRequiredScopeRule {
    fn check(&self, item: &LenientTokenProfile) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenProfileRequiredNameRule {
    pub message: String,
}

impl LenientTokenProfileRule for LenientTokenProfileRequiredNameRule {
    fn check(&self, item: &LenientTokenProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenProfileAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientTokenProfileRule for LenientTokenProfileAmountRangeRule {
    fn check(&self, item: &LenientTokenProfile) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient token profile.
#[derive(Debug, Clone)]
pub struct LenientTokenProfileValidator {
    pub rules: Vec<Rc<dyn LenientTokenProfileRule>>,
}

impl LenientTokenProfileValidator {
    pub fn validate(&self, item: &LenientTokenProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientTokenProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTokenProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_token_profile_validator() -> LenientTokenProfileValidator {
    LenientTokenProfileValidator {
        rules: vec![
            Rc::new(LenientTokenProfileRequiredCodeRule {
                message: "lenient token profile needs a value".to_string(),
            }) as Rc<dyn LenientTokenProfileRule>,
            Rc::new(LenientTokenProfileActiveGuardRule { limit: 3600 }),
            Rc::new(LenientTokenProfileNameLengthRule { max_length: 80 }),
            Rc::new(LenientTokenProfileRequiredScopeRule {
                message: "lenient token profile needs this field".to_string(),
            }),
            Rc::new(LenientTokenProfileRequiredNameRule {
                message: "lenient token profile needs this field".to_string(),
            }),
            Rc::new(LenientTokenProfileAmountRangeRule { min: 5.8, max: 1374.8 }),
        ],
    }
}
