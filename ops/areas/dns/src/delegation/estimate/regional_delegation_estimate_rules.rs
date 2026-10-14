use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional delegation estimate must pass. Check returns its problems, or none.
pub trait RegionalDelegationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDelegationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationEstimateDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalDelegationEstimateRule for RegionalDelegationEstimateDurationMinutesRangeRule {
    fn check(&self, item: &RegionalDelegationEstimate) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationEstimateRequiredPrincipalRule {
    pub message: String,
}

impl RegionalDelegationEstimateRule for RegionalDelegationEstimateRequiredPrincipalRule {
    fn check(&self, item: &RegionalDelegationEstimate) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationEstimateCodePrefixRule {
    pub prefix: String,
}

impl RegionalDelegationEstimateRule for RegionalDelegationEstimateCodePrefixRule {
    fn check(&self, item: &RegionalDelegationEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationEstimateRequiredCodeRule {
    pub message: String,
}

impl RegionalDelegationEstimateRule for RegionalDelegationEstimateRequiredCodeRule {
    fn check(&self, item: &RegionalDelegationEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional delegation estimate.
#[derive(Debug, Clone)]
pub struct RegionalDelegationEstimateValidator {
    pub rules: Vec<Rc<dyn RegionalDelegationEstimateRule>>,
}

impl RegionalDelegationEstimateValidator {
    pub fn validate(&self, item: &RegionalDelegationEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalDelegationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDelegationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_delegation_estimate_validator() -> RegionalDelegationEstimateValidator {
    RegionalDelegationEstimateValidator {
        rules: vec![
            Rc::new(RegionalDelegationEstimateDurationMinutesRangeRule { min: 1000, max: 1000 })
                as Rc<dyn RegionalDelegationEstimateRule>,
            Rc::new(RegionalDelegationEstimateRequiredPrincipalRule {
                message: "regional delegation estimate needs an entry".to_string(),
            }),
            Rc::new(RegionalDelegationEstimateCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RegionalDelegationEstimateRequiredCodeRule {
                message: "regional delegation estimate needs this field".to_string(),
            }),
        ],
    }
}
