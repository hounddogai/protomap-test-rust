use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient nonce variance must pass. Check returns its problems, or none.
pub trait LenientNonceVarianceRule: std::fmt::Debug {
    fn check(&self, item: &LenientNonceVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceVarianceNameLengthRule {
    pub max_length: i64,
}

impl LenientNonceVarianceRule for LenientNonceVarianceNameLengthRule {
    fn check(&self, item: &LenientNonceVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceVarianceCodePrefixRule {
    pub prefix: String,
}

impl LenientNonceVarianceRule for LenientNonceVarianceCodePrefixRule {
    fn check(&self, item: &LenientNonceVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceVarianceAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl LenientNonceVarianceRule for LenientNonceVarianceAllowedOwnerIdRule {
    fn check(&self, item: &LenientNonceVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is stale", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceVarianceRequiredCodeRule {
    pub message: String,
}

impl LenientNonceVarianceRule for LenientNonceVarianceRequiredCodeRule {
    fn check(&self, item: &LenientNonceVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceVarianceAudienceLengthRule {
    pub max_length: i64,
}

impl LenientNonceVarianceRule for LenientNonceVarianceAudienceLengthRule {
    fn check(&self, item: &LenientNonceVariance) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient nonce variance.
#[derive(Debug, Clone)]
pub struct LenientNonceVarianceValidator {
    pub rules: Vec<Rc<dyn LenientNonceVarianceRule>>,
}

impl LenientNonceVarianceValidator {
    pub fn validate(&self, item: &LenientNonceVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientNonceVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientNonceVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_nonce_variance_validator() -> LenientNonceVarianceValidator {
    LenientNonceVarianceValidator {
        rules: vec![
            Rc::new(LenientNonceVarianceNameLengthRule { max_length: 255 }) as Rc<dyn LenientNonceVarianceRule>,
            Rc::new(LenientNonceVarianceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LenientNonceVarianceAllowedOwnerIdRule {
                allowed: vec!["pending".to_string(), "adaptive".to_string(), "primary".to_string()],
            }),
            Rc::new(LenientNonceVarianceRequiredCodeRule {
                message: "lenient nonce variance needs an entry".to_string(),
            }),
            Rc::new(LenientNonceVarianceAudienceLengthRule { max_length: 120 }),
        ],
    }
}
