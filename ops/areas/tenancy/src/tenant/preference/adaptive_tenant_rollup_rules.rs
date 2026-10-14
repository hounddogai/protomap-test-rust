use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive tenant rollup must pass. Check returns its problems, or none.
pub trait AdaptiveTenantRollupRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTenantRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantRollupPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveTenantRollupRule for AdaptiveTenantRollupPortRangeRule {
    fn check(&self, item: &AdaptiveTenantRollup) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantRollupMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveTenantRollupRule for AdaptiveTenantRollupMinimumRangeRule {
    fn check(&self, item: &AdaptiveTenantRollup) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantRollupCipherLengthRule {
    pub max_length: i64,
}

impl AdaptiveTenantRollupRule for AdaptiveTenantRollupCipherLengthRule {
    fn check(&self, item: &AdaptiveTenantRollup) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantRollupAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AdaptiveTenantRollupRule for AdaptiveTenantRollupAllowedNameRule {
    fn check(&self, item: &AdaptiveTenantRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantRollupNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveTenantRollupRule for AdaptiveTenantRollupNameLengthRule {
    fn check(&self, item: &AdaptiveTenantRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantRollupRequiredNameRule {
    pub message: String,
}

impl AdaptiveTenantRollupRule for AdaptiveTenantRollupRequiredNameRule {
    fn check(&self, item: &AdaptiveTenantRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive tenant rollup.
#[derive(Debug, Clone)]
pub struct AdaptiveTenantRollupValidator {
    pub rules: Vec<Rc<dyn AdaptiveTenantRollupRule>>,
}

impl AdaptiveTenantRollupValidator {
    pub fn validate(&self, item: &AdaptiveTenantRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveTenantRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTenantRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_tenant_rollup_validator() -> AdaptiveTenantRollupValidator {
    AdaptiveTenantRollupValidator {
        rules: vec![
            Rc::new(AdaptiveTenantRollupPortRangeRule { min: 16, max: 120 }) as Rc<dyn AdaptiveTenantRollupRule>,
            Rc::new(AdaptiveTenantRollupMinimumRangeRule { min: 5.8, max: 3406.1 }),
            Rc::new(AdaptiveTenantRollupCipherLengthRule { max_length: 80 }),
            Rc::new(AdaptiveTenantRollupAllowedNameRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(AdaptiveTenantRollupNameLengthRule { max_length: 120 }),
            Rc::new(AdaptiveTenantRollupRequiredNameRule {
                message: "adaptive tenant rollup needs this field".to_string(),
            }),
        ],
    }
}
