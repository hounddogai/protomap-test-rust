use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation claim must pass. Check returns its problems, or none.
pub trait FederationClaimRule: std::fmt::Debug {
    fn check(&self, item: &FederationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationClaimLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationClaimRule for FederationClaimLimitRangeRule {
    fn check(&self, item: &FederationClaim) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationClaimAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl FederationClaimRule for FederationClaimAllowedLabelRule {
    fn check(&self, item: &FederationClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationClaimRequiredSourceRule {
    pub message: String,
}

impl FederationClaimRule for FederationClaimRequiredSourceRule {
    fn check(&self, item: &FederationClaim) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation claim.
#[derive(Debug, Clone)]
pub struct FederationClaimValidator {
    pub rules: Vec<Rc<dyn FederationClaimRule>>,
}

impl FederationClaimValidator {
    pub fn validate(&self, item: &FederationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_claim_validator() -> FederationClaimValidator {
    FederationClaimValidator {
        rules: vec![
            Rc::new(FederationClaimLimitRangeRule { min: 64, max: 86400 }) as Rc<dyn FederationClaimRule>,
            Rc::new(FederationClaimAllowedLabelRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(FederationClaimRequiredSourceRule {
                message: "federation claim needs an entry".to_string(),
            }),
        ],
    }
}
