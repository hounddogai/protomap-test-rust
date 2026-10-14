use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant rollup must pass. Check returns its problems, or none.
pub trait TenantRollupRule: std::fmt::Debug {
    fn check(&self, item: &TenantRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRollupCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantRollupRule for TenantRollupCostRangeRule {
    fn check(&self, item: &TenantRollup) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRollupMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantRollupRule for TenantRollupMaximumRangeRule {
    fn check(&self, item: &TenantRollup) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRollupCodeLengthRule {
    pub max_length: i64,
}

impl TenantRollupRule for TenantRollupCodeLengthRule {
    fn check(&self, item: &TenantRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRollupTitleLengthRule {
    pub max_length: i64,
}

impl TenantRollupRule for TenantRollupTitleLengthRule {
    fn check(&self, item: &TenantRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRollupFingerprintLengthRule {
    pub max_length: i64,
}

impl TenantRollupRule for TenantRollupFingerprintLengthRule {
    fn check(&self, item: &TenantRollup) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantRollupReviewerLengthRule {
    pub max_length: i64,
}

impl TenantRollupRule for TenantRollupReviewerLengthRule {
    fn check(&self, item: &TenantRollup) -> Vec<String> {
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

/// Runs every rule of a tenant rollup.
#[derive(Debug, Clone)]
pub struct TenantRollupValidator {
    pub rules: Vec<Rc<dyn TenantRollupRule>>,
}

impl TenantRollupValidator {
    pub fn validate(&self, item: &TenantRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_rollup_validator() -> TenantRollupValidator {
    TenantRollupValidator {
        rules: vec![
            Rc::new(TenantRollupCostRangeRule { min: 5.8, max: 2176.6 }) as Rc<dyn TenantRollupRule>,
            Rc::new(TenantRollupMaximumRangeRule { min: 1.0, max: 528.7 }),
            Rc::new(TenantRollupCodeLengthRule { max_length: 3600 }),
            Rc::new(TenantRollupTitleLengthRule { max_length: 16 }),
            Rc::new(TenantRollupFingerprintLengthRule { max_length: 3600 }),
            Rc::new(TenantRollupReviewerLengthRule { max_length: 64 }),
        ],
    }
}
