use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional federation adjustment must pass. Check returns its problems, or none.
pub trait RegionalFederationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RegionalFederationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl RegionalFederationAdjustmentRule for RegionalFederationAdjustmentCodeLengthRule {
    fn check(&self, item: &RegionalFederationAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAdjustmentIssuerLengthRule {
    pub max_length: i64,
}

impl RegionalFederationAdjustmentRule for RegionalFederationAdjustmentIssuerLengthRule {
    fn check(&self, item: &RegionalFederationAdjustment) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAdjustmentEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalFederationAdjustmentRule for RegionalFederationAdjustmentEndsAtRangeRule {
    fn check(&self, item: &RegionalFederationAdjustment) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a regional federation adjustment.
#[derive(Debug, Clone)]
pub struct RegionalFederationAdjustmentValidator {
    pub rules: Vec<Rc<dyn RegionalFederationAdjustmentRule>>,
}

impl RegionalFederationAdjustmentValidator {
    pub fn validate(&self, item: &RegionalFederationAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalFederationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalFederationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_federation_adjustment_validator() -> RegionalFederationAdjustmentValidator {
    RegionalFederationAdjustmentValidator {
        rules: vec![
            Rc::new(RegionalFederationAdjustmentCodeLengthRule { max_length: 64 })
                as Rc<dyn RegionalFederationAdjustmentRule>,
            Rc::new(RegionalFederationAdjustmentIssuerLengthRule { max_length: 64 }),
            Rc::new(RegionalFederationAdjustmentEndsAtRangeRule { min: 32, max: 120 }),
        ],
    }
}
