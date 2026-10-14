use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension milestone must pass. Check returns its problems, or none.
pub trait SuspensionMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMilestoneRequiredLabelRule {
    pub message: String,
}

impl SuspensionMilestoneRule for SuspensionMilestoneRequiredLabelRule {
    fn check(&self, item: &SuspensionMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMilestoneRequiredCodeRule {
    pub message: String,
}

impl SuspensionMilestoneRule for SuspensionMilestoneRequiredCodeRule {
    fn check(&self, item: &SuspensionMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMilestoneAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SuspensionMilestoneRule for SuspensionMilestoneAllowedCodeRule {
    fn check(&self, item: &SuspensionMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMilestoneCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionMilestoneRule for SuspensionMilestoneCapacityRangeRule {
    fn check(&self, item: &SuspensionMilestone) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMilestoneCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionMilestoneRule for SuspensionMilestoneCostRangeRule {
    fn check(&self, item: &SuspensionMilestone) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMilestoneBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionMilestoneRule for SuspensionMilestoneBurstRangeRule {
    fn check(&self, item: &SuspensionMilestone) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension milestone.
#[derive(Debug, Clone)]
pub struct SuspensionMilestoneValidator {
    pub rules: Vec<Rc<dyn SuspensionMilestoneRule>>,
}

impl SuspensionMilestoneValidator {
    pub fn validate(&self, item: &SuspensionMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_milestone_validator() -> SuspensionMilestoneValidator {
    SuspensionMilestoneValidator {
        rules: vec![
            Rc::new(SuspensionMilestoneRequiredLabelRule {
                message: "suspension milestone needs a value".to_string(),
            }) as Rc<dyn SuspensionMilestoneRule>,
            Rc::new(SuspensionMilestoneRequiredCodeRule {
                message: "suspension milestone needs a value".to_string(),
            }),
            Rc::new(SuspensionMilestoneAllowedCodeRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(SuspensionMilestoneCapacityRangeRule { min: 64, max: 255 }),
            Rc::new(SuspensionMilestoneCostRangeRule { min: 0.2, max: 1648.4 }),
            Rc::new(SuspensionMilestoneBurstRangeRule { min: 16, max: 1000 }),
        ],
    }
}
