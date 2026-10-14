use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual federation target must pass. Check returns its problems, or none.
pub trait ManualFederationTargetRule: std::fmt::Debug {
    fn check(&self, item: &ManualFederationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationTargetDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualFederationTargetRule for ManualFederationTargetDiscountRangeRule {
    fn check(&self, item: &ManualFederationTarget) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationTargetRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualFederationTargetRule for ManualFederationTargetRevisionRangeRule {
    fn check(&self, item: &ManualFederationTarget) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationTargetCodeLengthRule {
    pub max_length: i64,
}

impl ManualFederationTargetRule for ManualFederationTargetCodeLengthRule {
    fn check(&self, item: &ManualFederationTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationTargetRequiredCodeRule {
    pub message: String,
}

impl ManualFederationTargetRule for ManualFederationTargetRequiredCodeRule {
    fn check(&self, item: &ManualFederationTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationTargetNameLengthRule {
    pub max_length: i64,
}

impl ManualFederationTargetRule for ManualFederationTargetNameLengthRule {
    fn check(&self, item: &ManualFederationTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual federation target.
#[derive(Debug, Clone)]
pub struct ManualFederationTargetValidator {
    pub rules: Vec<Rc<dyn ManualFederationTargetRule>>,
}

impl ManualFederationTargetValidator {
    pub fn validate(&self, item: &ManualFederationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualFederationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualFederationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_federation_target_validator() -> ManualFederationTargetValidator {
    ManualFederationTargetValidator {
        rules: vec![
            Rc::new(ManualFederationTargetDiscountRangeRule { min: 2.6, max: 894.0 })
                as Rc<dyn ManualFederationTargetRule>,
            Rc::new(ManualFederationTargetRevisionRangeRule { min: 16, max: 3600 }),
            Rc::new(ManualFederationTargetCodeLengthRule { max_length: 80 }),
            Rc::new(ManualFederationTargetRequiredCodeRule {
                message: "manual federation target needs a value".to_string(),
            }),
            Rc::new(ManualFederationTargetNameLengthRule { max_length: 64 }),
        ],
    }
}
