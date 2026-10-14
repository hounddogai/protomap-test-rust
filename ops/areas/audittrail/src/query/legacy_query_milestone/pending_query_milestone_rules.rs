use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending query milestone must pass. Check returns its problems, or none.
pub trait PendingQueryMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &PendingQueryMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMilestoneBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingQueryMilestoneRule for PendingQueryMilestoneBurstRangeRule {
    fn check(&self, item: &PendingQueryMilestone) -> Vec<String> {
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
pub struct PendingQueryMilestoneRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingQueryMilestoneRule for PendingQueryMilestoneRatioRangeRule {
    fn check(&self, item: &PendingQueryMilestone) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMilestoneAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingQueryMilestoneRule for PendingQueryMilestoneAllowedLabelRule {
    fn check(&self, item: &PendingQueryMilestone) -> Vec<String> {
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
pub struct PendingQueryMilestoneQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingQueryMilestoneRule for PendingQueryMilestoneQuotaUnitsRangeRule {
    fn check(&self, item: &PendingQueryMilestone) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMilestoneRequiredLabelRule {
    pub message: String,
}

impl PendingQueryMilestoneRule for PendingQueryMilestoneRequiredLabelRule {
    fn check(&self, item: &PendingQueryMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMilestoneLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingQueryMilestoneRule for PendingQueryMilestoneLimitRangeRule {
    fn check(&self, item: &PendingQueryMilestone) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending query milestone.
#[derive(Debug, Clone)]
pub struct PendingQueryMilestoneValidator {
    pub rules: Vec<Rc<dyn PendingQueryMilestoneRule>>,
}

impl PendingQueryMilestoneValidator {
    pub fn validate(&self, item: &PendingQueryMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingQueryMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingQueryMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_query_milestone_validator() -> PendingQueryMilestoneValidator {
    PendingQueryMilestoneValidator {
        rules: vec![
            Rc::new(PendingQueryMilestoneBurstRangeRule { min: 3600, max: 86400 }) as Rc<dyn PendingQueryMilestoneRule>,
            Rc::new(PendingQueryMilestoneRatioRangeRule { min: 4.9, max: 464.9 }),
            Rc::new(PendingQueryMilestoneAllowedLabelRule {
                allowed: vec!["fallback".to_string(), "rolling".to_string()],
            }),
            Rc::new(PendingQueryMilestoneQuotaUnitsRangeRule { min: 64, max: 1000 }),
            Rc::new(PendingQueryMilestoneRequiredLabelRule {
                message: "pending query milestone needs a value".to_string(),
            }),
            Rc::new(PendingQueryMilestoneLimitRangeRule { min: 120, max: 86400 }),
        ],
    }
}
