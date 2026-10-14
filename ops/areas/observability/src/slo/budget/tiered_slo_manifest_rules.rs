use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered slo manifest must pass. Check returns its problems, or none.
pub trait TieredSloManifestRule: std::fmt::Debug {
    fn check(&self, item: &TieredSloManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloManifestRequiredScopeRule {
    pub message: String,
}

impl TieredSloManifestRule for TieredSloManifestRequiredScopeRule {
    fn check(&self, item: &TieredSloManifest) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloManifestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredSloManifestRule for TieredSloManifestCeilingRangeRule {
    fn check(&self, item: &TieredSloManifest) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloManifestCodeLengthRule {
    pub max_length: i64,
}

impl TieredSloManifestRule for TieredSloManifestCodeLengthRule {
    fn check(&self, item: &TieredSloManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloManifestMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredSloManifestRule for TieredSloManifestMarginRangeRule {
    fn check(&self, item: &TieredSloManifest) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloManifestRequiredCurrencyRule {
    pub message: String,
}

impl TieredSloManifestRule for TieredSloManifestRequiredCurrencyRule {
    fn check(&self, item: &TieredSloManifest) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloManifestStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredSloManifestRule for TieredSloManifestStartsAtRangeRule {
    fn check(&self, item: &TieredSloManifest) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered slo manifest.
#[derive(Debug, Clone)]
pub struct TieredSloManifestValidator {
    pub rules: Vec<Rc<dyn TieredSloManifestRule>>,
}

impl TieredSloManifestValidator {
    pub fn validate(&self, item: &TieredSloManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredSloManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredSloManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_slo_manifest_validator() -> TieredSloManifestValidator {
    TieredSloManifestValidator {
        rules: vec![
            Rc::new(TieredSloManifestRequiredScopeRule {
                message: "tiered slo manifest needs this field".to_string(),
            }) as Rc<dyn TieredSloManifestRule>,
            Rc::new(TieredSloManifestCeilingRangeRule { min: 6.7, max: 2322.4 }),
            Rc::new(TieredSloManifestCodeLengthRule { max_length: 1000 }),
            Rc::new(TieredSloManifestMarginRangeRule { min: 4.8, max: 3417.3 }),
            Rc::new(TieredSloManifestRequiredCurrencyRule {
                message: "tiered slo manifest needs a value".to_string(),
            }),
            Rc::new(TieredSloManifestStartsAtRangeRule { min: 80, max: 86400 }),
        ],
    }
}
