use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin policy must pass. Check returns its problems, or none.
pub trait RegionPinPolicyRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPolicyQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinPolicyRule for RegionPinPolicyQuantityRangeRule {
    fn check(&self, item: &RegionPinPolicy) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPolicyQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinPolicyRule for RegionPinPolicyQuotaUnitsRangeRule {
    fn check(&self, item: &RegionPinPolicy) -> Vec<String> {
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
pub struct RegionPinPolicyRequiredCodeRule {
    pub message: String,
}

impl RegionPinPolicyRule for RegionPinPolicyRequiredCodeRule {
    fn check(&self, item: &RegionPinPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin policy.
#[derive(Debug, Clone)]
pub struct RegionPinPolicyValidator {
    pub rules: Vec<Rc<dyn RegionPinPolicyRule>>,
}

impl RegionPinPolicyValidator {
    pub fn validate(&self, item: &RegionPinPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_policy_validator() -> RegionPinPolicyValidator {
    RegionPinPolicyValidator {
        rules: vec![
            Rc::new(RegionPinPolicyQuantityRangeRule { min: 80, max: 1000 }) as Rc<dyn RegionPinPolicyRule>,
            Rc::new(RegionPinPolicyQuotaUnitsRangeRule { min: 64, max: 255 }),
            Rc::new(RegionPinPolicyRequiredCodeRule {
                message: "region pin policy needs an entry".to_string(),
            }),
        ],
    }
}
