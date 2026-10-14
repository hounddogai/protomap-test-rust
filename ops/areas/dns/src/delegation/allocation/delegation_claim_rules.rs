use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation claim must pass. Check returns its problems, or none.
pub trait DelegationClaimRule: std::fmt::Debug {
    fn check(&self, item: &DelegationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationClaimBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationClaimRule for DelegationClaimBurstRangeRule {
    fn check(&self, item: &DelegationClaim) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationClaimApprovedGuardRule {
    pub limit: f64,
}

impl DelegationClaimRule for DelegationClaimApprovedGuardRule {
    fn check(&self, item: &DelegationClaim) -> Vec<String> {
        if item.approved && (item.ceiling > self.limit) {
            return vec![format!(
                "ceiling {} exceeds {} while approved",
                item.ceiling, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationClaimNameLengthRule {
    pub max_length: i64,
}

impl DelegationClaimRule for DelegationClaimNameLengthRule {
    fn check(&self, item: &DelegationClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationClaimCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationClaimRule for DelegationClaimCapacityRangeRule {
    fn check(&self, item: &DelegationClaim) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationClaimCodeLengthRule {
    pub max_length: i64,
}

impl DelegationClaimRule for DelegationClaimCodeLengthRule {
    fn check(&self, item: &DelegationClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation claim.
#[derive(Debug, Clone)]
pub struct DelegationClaimValidator {
    pub rules: Vec<Rc<dyn DelegationClaimRule>>,
}

impl DelegationClaimValidator {
    pub fn validate(&self, item: &DelegationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_claim_validator() -> DelegationClaimValidator {
    DelegationClaimValidator {
        rules: vec![
            Rc::new(DelegationClaimBurstRangeRule { min: 1000, max: 86400 }) as Rc<dyn DelegationClaimRule>,
            Rc::new(DelegationClaimApprovedGuardRule { limit: 5.0 }),
            Rc::new(DelegationClaimNameLengthRule { max_length: 32 }),
            Rc::new(DelegationClaimCapacityRangeRule { min: 80, max: 3600 }),
            Rc::new(DelegationClaimCodeLengthRule { max_length: 1000 }),
        ],
    }
}
