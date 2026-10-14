use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan segment must pass. Check returns its problems, or none.
pub trait PlanSegmentRule: std::fmt::Debug {
    fn check(&self, item: &PlanSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSegmentAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl PlanSegmentRule for PlanSegmentAllowedPhoneRule {
    fn check(&self, item: &PlanSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is unknown", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSegmentRequiredCodeRule {
    pub message: String,
}

impl PlanSegmentRule for PlanSegmentRequiredCodeRule {
    fn check(&self, item: &PlanSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSegmentRequiredKeyIdRule {
    pub message: String,
}

impl PlanSegmentRule for PlanSegmentRequiredKeyIdRule {
    fn check(&self, item: &PlanSegment) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSegmentRequiredPhoneRule {
    pub message: String,
}

impl PlanSegmentRule for PlanSegmentRequiredPhoneRule {
    fn check(&self, item: &PlanSegment) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSegmentQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanSegmentRule for PlanSegmentQuotaUnitsRangeRule {
    fn check(&self, item: &PlanSegment) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan segment.
#[derive(Debug, Clone)]
pub struct PlanSegmentValidator {
    pub rules: Vec<Rc<dyn PlanSegmentRule>>,
}

impl PlanSegmentValidator {
    pub fn validate(&self, item: &PlanSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_segment_validator() -> PlanSegmentValidator {
    PlanSegmentValidator {
        rules: vec![
            Rc::new(PlanSegmentAllowedPhoneRule { allowed: Vec::new() }) as Rc<dyn PlanSegmentRule>,
            Rc::new(PlanSegmentRequiredCodeRule {
                message: "plan segment needs an entry".to_string(),
            }),
            Rc::new(PlanSegmentRequiredKeyIdRule {
                message: "plan segment needs a value".to_string(),
            }),
            Rc::new(PlanSegmentRequiredPhoneRule {
                message: "plan segment needs this field".to_string(),
            }),
            Rc::new(PlanSegmentQuotaUnitsRangeRule { min: 16, max: 86400 }),
        ],
    }
}
