use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global rollout claim must pass. Check returns its problems, or none.
pub trait GlobalRolloutClaimRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRolloutClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutClaimRequiredCodeRule {
    pub message: String,
}

impl GlobalRolloutClaimRule for GlobalRolloutClaimRequiredCodeRule {
    fn check(&self, item: &GlobalRolloutClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutClaimCountryLengthRule {
    pub max_length: i64,
}

impl GlobalRolloutClaimRule for GlobalRolloutClaimCountryLengthRule {
    fn check(&self, item: &GlobalRolloutClaim) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutClaimRequiredSummaryRule {
    pub message: String,
}

impl GlobalRolloutClaimRule for GlobalRolloutClaimRequiredSummaryRule {
    fn check(&self, item: &GlobalRolloutClaim) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutClaimSummaryLengthRule {
    pub max_length: i64,
}

impl GlobalRolloutClaimRule for GlobalRolloutClaimSummaryLengthRule {
    fn check(&self, item: &GlobalRolloutClaim) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutClaimAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl GlobalRolloutClaimRule for GlobalRolloutClaimAllowedCountryRule {
    fn check(&self, item: &GlobalRolloutClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is stale", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutClaimDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalRolloutClaimRule for GlobalRolloutClaimDurationMinutesRangeRule {
    fn check(&self, item: &GlobalRolloutClaim) -> Vec<String> {
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

/// Runs every rule of a global rollout claim.
#[derive(Debug, Clone)]
pub struct GlobalRolloutClaimValidator {
    pub rules: Vec<Rc<dyn GlobalRolloutClaimRule>>,
}

impl GlobalRolloutClaimValidator {
    pub fn validate(&self, item: &GlobalRolloutClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalRolloutClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRolloutClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_rollout_claim_validator() -> GlobalRolloutClaimValidator {
    GlobalRolloutClaimValidator {
        rules: vec![
            Rc::new(GlobalRolloutClaimRequiredCodeRule {
                message: "global rollout claim needs a value".to_string(),
            }) as Rc<dyn GlobalRolloutClaimRule>,
            Rc::new(GlobalRolloutClaimCountryLengthRule { max_length: 3600 }),
            Rc::new(GlobalRolloutClaimRequiredSummaryRule {
                message: "global rollout claim needs a value".to_string(),
            }),
            Rc::new(GlobalRolloutClaimSummaryLengthRule { max_length: 86400 }),
            Rc::new(GlobalRolloutClaimAllowedCountryRule { allowed: Vec::new() }),
            Rc::new(GlobalRolloutClaimDurationMinutesRangeRule { min: 1000, max: 3600 }),
        ],
    }
}
