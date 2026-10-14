use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy peering estimate must pass. Check returns its problems, or none.
pub trait LegacyPeeringEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPeeringEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringEstimateCodePrefixRule {
    pub prefix: String,
}

impl LegacyPeeringEstimateRule for LegacyPeeringEstimateCodePrefixRule {
    fn check(&self, item: &LegacyPeeringEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringEstimateMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyPeeringEstimateRule for LegacyPeeringEstimateMarginRangeRule {
    fn check(&self, item: &LegacyPeeringEstimate) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringEstimateRequiredTitleRule {
    pub message: String,
}

impl LegacyPeeringEstimateRule for LegacyPeeringEstimateRequiredTitleRule {
    fn check(&self, item: &LegacyPeeringEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy peering estimate.
#[derive(Debug, Clone)]
pub struct LegacyPeeringEstimateValidator {
    pub rules: Vec<Rc<dyn LegacyPeeringEstimateRule>>,
}

impl LegacyPeeringEstimateValidator {
    pub fn validate(&self, item: &LegacyPeeringEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyPeeringEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPeeringEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_peering_estimate_validator() -> LegacyPeeringEstimateValidator {
    LegacyPeeringEstimateValidator {
        rules: vec![
            Rc::new(LegacyPeeringEstimateCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn LegacyPeeringEstimateRule>,
            Rc::new(LegacyPeeringEstimateMarginRangeRule { min: 4.5, max: 139.2 }),
            Rc::new(LegacyPeeringEstimateRequiredTitleRule {
                message: "legacy peering estimate needs an entry".to_string(),
            }),
        ],
    }
}
