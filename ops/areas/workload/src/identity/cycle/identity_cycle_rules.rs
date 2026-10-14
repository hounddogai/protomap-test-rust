use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity cycle must pass. Check returns its problems, or none.
pub trait IdentityCycleRule: std::fmt::Debug {
    fn check(&self, item: &IdentityCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCycleTenantIdLengthRule {
    pub max_length: i64,
}

impl IdentityCycleRule for IdentityCycleTenantIdLengthRule {
    fn check(&self, item: &IdentityCycle) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCycleCodePrefixRule {
    pub prefix: String,
}

impl IdentityCycleRule for IdentityCycleCodePrefixRule {
    fn check(&self, item: &IdentityCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCycleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityCycleRule for IdentityCycleQuotaUnitsRangeRule {
    fn check(&self, item: &IdentityCycle) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCycleRequiredFingerprintRule {
    pub message: String,
}

impl IdentityCycleRule for IdentityCycleRequiredFingerprintRule {
    fn check(&self, item: &IdentityCycle) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCycleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IdentityCycleRule for IdentityCycleMaximumRangeRule {
    fn check(&self, item: &IdentityCycle) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCycleNameLengthRule {
    pub max_length: i64,
}

impl IdentityCycleRule for IdentityCycleNameLengthRule {
    fn check(&self, item: &IdentityCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity cycle.
#[derive(Debug, Clone)]
pub struct IdentityCycleValidator {
    pub rules: Vec<Rc<dyn IdentityCycleRule>>,
}

impl IdentityCycleValidator {
    pub fn validate(&self, item: &IdentityCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_cycle_validator() -> IdentityCycleValidator {
    IdentityCycleValidator {
        rules: vec![
            Rc::new(IdentityCycleTenantIdLengthRule { max_length: 32 }) as Rc<dyn IdentityCycleRule>,
            Rc::new(IdentityCycleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(IdentityCycleQuotaUnitsRangeRule { min: 64, max: 64 }),
            Rc::new(IdentityCycleRequiredFingerprintRule {
                message: "identity cycle needs an entry".to_string(),
            }),
            Rc::new(IdentityCycleMaximumRangeRule { min: 6.4, max: 4617.5 }),
            Rc::new(IdentityCycleNameLengthRule { max_length: 3600 }),
        ],
    }
}
