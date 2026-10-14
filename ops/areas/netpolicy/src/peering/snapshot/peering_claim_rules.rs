use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering claim must pass. Check returns its problems, or none.
pub trait PeeringClaimRule: std::fmt::Debug {
    fn check(&self, item: &PeeringClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringClaimLabelLengthRule {
    pub max_length: i64,
}

impl PeeringClaimRule for PeeringClaimLabelLengthRule {
    fn check(&self, item: &PeeringClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringClaimRequiredDescriptionRule {
    pub message: String,
}

impl PeeringClaimRule for PeeringClaimRequiredDescriptionRule {
    fn check(&self, item: &PeeringClaim) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringClaimPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringClaimRule for PeeringClaimPriorityRangeRule {
    fn check(&self, item: &PeeringClaim) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringClaimCodeLengthRule {
    pub max_length: i64,
}

impl PeeringClaimRule for PeeringClaimCodeLengthRule {
    fn check(&self, item: &PeeringClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering claim.
#[derive(Debug, Clone)]
pub struct PeeringClaimValidator {
    pub rules: Vec<Rc<dyn PeeringClaimRule>>,
}

impl PeeringClaimValidator {
    pub fn validate(&self, item: &PeeringClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_claim_validator() -> PeeringClaimValidator {
    PeeringClaimValidator {
        rules: vec![
            Rc::new(PeeringClaimLabelLengthRule { max_length: 86400 }) as Rc<dyn PeeringClaimRule>,
            Rc::new(PeeringClaimRequiredDescriptionRule {
                message: "peering claim needs a value".to_string(),
            }),
            Rc::new(PeeringClaimPriorityRangeRule { min: 3600, max: 3600 }),
            Rc::new(PeeringClaimCodeLengthRule { max_length: 255 }),
        ],
    }
}
