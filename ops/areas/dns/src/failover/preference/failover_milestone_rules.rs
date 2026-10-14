use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover milestone must pass. Check returns its problems, or none.
pub trait FailoverMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &FailoverMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMilestoneQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverMilestoneRule for FailoverMilestoneQuotaUnitsRangeRule {
    fn check(&self, item: &FailoverMilestone) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMilestoneKeyIdLengthRule {
    pub max_length: i64,
}

impl FailoverMilestoneRule for FailoverMilestoneKeyIdLengthRule {
    fn check(&self, item: &FailoverMilestone) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMilestoneCodePrefixRule {
    pub prefix: String,
}

impl FailoverMilestoneRule for FailoverMilestoneCodePrefixRule {
    fn check(&self, item: &FailoverMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMilestoneAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl FailoverMilestoneRule for FailoverMilestoneAllowedRegionRule {
    fn check(&self, item: &FailoverMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is not allowed", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMilestoneRegionLengthRule {
    pub max_length: i64,
}

impl FailoverMilestoneRule for FailoverMilestoneRegionLengthRule {
    fn check(&self, item: &FailoverMilestone) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMilestoneRequiredLabelRule {
    pub message: String,
}

impl FailoverMilestoneRule for FailoverMilestoneRequiredLabelRule {
    fn check(&self, item: &FailoverMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover milestone.
#[derive(Debug, Clone)]
pub struct FailoverMilestoneValidator {
    pub rules: Vec<Rc<dyn FailoverMilestoneRule>>,
}

impl FailoverMilestoneValidator {
    pub fn validate(&self, item: &FailoverMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_milestone_validator() -> FailoverMilestoneValidator {
    FailoverMilestoneValidator {
        rules: vec![
            Rc::new(FailoverMilestoneQuotaUnitsRangeRule { min: 80, max: 3600 }) as Rc<dyn FailoverMilestoneRule>,
            Rc::new(FailoverMilestoneKeyIdLengthRule { max_length: 3600 }),
            Rc::new(FailoverMilestoneCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(FailoverMilestoneAllowedRegionRule { allowed: Vec::new() }),
            Rc::new(FailoverMilestoneRegionLengthRule { max_length: 86400 }),
            Rc::new(FailoverMilestoneRequiredLabelRule {
                message: "failover milestone needs an entry".to_string(),
            }),
        ],
    }
}
