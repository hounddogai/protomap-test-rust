use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling ttl policy proposal must pass. Check returns its problems, or none.
pub trait RollingTtlPolicyProposalRule: std::fmt::Debug {
    fn check(&self, item: &RollingTtlPolicyProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyProposalVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTtlPolicyProposalRule for RollingTtlPolicyProposalVersionRangeRule {
    fn check(&self, item: &RollingTtlPolicyProposal) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyProposalDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTtlPolicyProposalRule for RollingTtlPolicyProposalDurationMinutesRangeRule {
    fn check(&self, item: &RollingTtlPolicyProposal) -> Vec<String> {
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
pub struct RollingTtlPolicyProposalRequiredContactEmailRule {
    pub message: String,
}

impl RollingTtlPolicyProposalRule for RollingTtlPolicyProposalRequiredContactEmailRule {
    fn check(&self, item: &RollingTtlPolicyProposal) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyProposalLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTtlPolicyProposalRule for RollingTtlPolicyProposalLimitRangeRule {
    fn check(&self, item: &RollingTtlPolicyProposal) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyProposalRequiredUnitRule {
    pub message: String,
}

impl RollingTtlPolicyProposalRule for RollingTtlPolicyProposalRequiredUnitRule {
    fn check(&self, item: &RollingTtlPolicyProposal) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyProposalRequiredCodeRule {
    pub message: String,
}

impl RollingTtlPolicyProposalRule for RollingTtlPolicyProposalRequiredCodeRule {
    fn check(&self, item: &RollingTtlPolicyProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling ttl policy proposal.
#[derive(Debug, Clone)]
pub struct RollingTtlPolicyProposalValidator {
    pub rules: Vec<Rc<dyn RollingTtlPolicyProposalRule>>,
}

impl RollingTtlPolicyProposalValidator {
    pub fn validate(&self, item: &RollingTtlPolicyProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingTtlPolicyProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTtlPolicyProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_ttl_policy_proposal_validator() -> RollingTtlPolicyProposalValidator {
    RollingTtlPolicyProposalValidator {
        rules: vec![
            Rc::new(RollingTtlPolicyProposalVersionRangeRule { min: 32, max: 80 })
                as Rc<dyn RollingTtlPolicyProposalRule>,
            Rc::new(RollingTtlPolicyProposalDurationMinutesRangeRule { min: 120, max: 86400 }),
            Rc::new(RollingTtlPolicyProposalRequiredContactEmailRule {
                message: "rolling ttl policy proposal needs a value".to_string(),
            }),
            Rc::new(RollingTtlPolicyProposalLimitRangeRule { min: 3600, max: 86400 }),
            Rc::new(RollingTtlPolicyProposalRequiredUnitRule {
                message: "rolling ttl policy proposal needs this field".to_string(),
            }),
            Rc::new(RollingTtlPolicyProposalRequiredCodeRule {
                message: "rolling ttl policy proposal needs a value".to_string(),
            }),
        ],
    }
}
