use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation milestone must pass. Check returns its problems, or none.
pub trait DelegationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &DelegationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMilestoneContactEmailLengthRule {
    pub max_length: i64,
}

impl DelegationMilestoneRule for DelegationMilestoneContactEmailLengthRule {
    fn check(&self, item: &DelegationMilestone) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMilestoneTenantIdLengthRule {
    pub max_length: i64,
}

impl DelegationMilestoneRule for DelegationMilestoneTenantIdLengthRule {
    fn check(&self, item: &DelegationMilestone) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMilestoneAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl DelegationMilestoneRule for DelegationMilestoneAllowedChecksumRule {
    fn check(&self, item: &DelegationMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is blocked", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMilestoneMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationMilestoneRule for DelegationMilestoneMinimumRangeRule {
    fn check(&self, item: &DelegationMilestone) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMilestoneRequiredContactEmailRule {
    pub message: String,
}

impl DelegationMilestoneRule for DelegationMilestoneRequiredContactEmailRule {
    fn check(&self, item: &DelegationMilestone) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation milestone.
#[derive(Debug, Clone)]
pub struct DelegationMilestoneValidator {
    pub rules: Vec<Rc<dyn DelegationMilestoneRule>>,
}

impl DelegationMilestoneValidator {
    pub fn validate(&self, item: &DelegationMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_milestone_validator() -> DelegationMilestoneValidator {
    DelegationMilestoneValidator {
        rules: vec![
            Rc::new(DelegationMilestoneContactEmailLengthRule { max_length: 64 }) as Rc<dyn DelegationMilestoneRule>,
            Rc::new(DelegationMilestoneTenantIdLengthRule { max_length: 3600 }),
            Rc::new(DelegationMilestoneAllowedChecksumRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(DelegationMilestoneMinimumRangeRule { min: 2.3, max: 923.2 }),
            Rc::new(DelegationMilestoneRequiredContactEmailRule {
                message: "delegation milestone needs this field".to_string(),
            }),
        ],
    }
}
