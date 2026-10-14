use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token adjustment must pass. Check returns its problems, or none.
pub trait TokenAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TokenAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAdjustmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TokenAdjustmentRule for TokenAdjustmentAllowedCodeRule {
    fn check(&self, item: &TokenAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl TokenAdjustmentRule for TokenAdjustmentCodeLengthRule {
    fn check(&self, item: &TokenAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAdjustmentMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenAdjustmentRule for TokenAdjustmentMaximumRangeRule {
    fn check(&self, item: &TokenAdjustment) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl TokenAdjustmentRule for TokenAdjustmentCodePrefixRule {
    fn check(&self, item: &TokenAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAdjustmentActiveGuardRule {
    pub limit: f64,
}

impl TokenAdjustmentRule for TokenAdjustmentActiveGuardRule {
    fn check(&self, item: &TokenAdjustment) -> Vec<String> {
        if item.active && (item.margin > self.limit) {
            return vec![format!("margin {} exceeds {} while active", item.margin, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl TokenAdjustmentRule for TokenAdjustmentNameLengthRule {
    fn check(&self, item: &TokenAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token adjustment.
#[derive(Debug, Clone)]
pub struct TokenAdjustmentValidator {
    pub rules: Vec<Rc<dyn TokenAdjustmentRule>>,
}

impl TokenAdjustmentValidator {
    pub fn validate(&self, item: &TokenAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_adjustment_validator() -> TokenAdjustmentValidator {
    TokenAdjustmentValidator {
        rules: vec![
            Rc::new(TokenAdjustmentAllowedCodeRule {
                allowed: vec!["batched".to_string()],
            }) as Rc<dyn TokenAdjustmentRule>,
            Rc::new(TokenAdjustmentCodeLengthRule { max_length: 120 }),
            Rc::new(TokenAdjustmentMaximumRangeRule { min: 0.6, max: 2103.3 }),
            Rc::new(TokenAdjustmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TokenAdjustmentActiveGuardRule { limit: 0.1 }),
            Rc::new(TokenAdjustmentNameLengthRule { max_length: 86400 }),
        ],
    }
}
