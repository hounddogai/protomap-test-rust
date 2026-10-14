use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment plan must pass. Check returns its problems, or none.
pub trait SegmentPlanRule: std::fmt::Debug {
    fn check(&self, item: &SegmentPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPlanRegionLengthRule {
    pub max_length: i64,
}

impl SegmentPlanRule for SegmentPlanRegionLengthRule {
    fn check(&self, item: &SegmentPlan) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPlanRequiredCodeRule {
    pub message: String,
}

impl SegmentPlanRule for SegmentPlanRequiredCodeRule {
    fn check(&self, item: &SegmentPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPlanRequiredCountryRule {
    pub message: String,
}

impl SegmentPlanRule for SegmentPlanRequiredCountryRule {
    fn check(&self, item: &SegmentPlan) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPlanCodePrefixRule {
    pub prefix: String,
}

impl SegmentPlanRule for SegmentPlanCodePrefixRule {
    fn check(&self, item: &SegmentPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPlanRequiredRegionRule {
    pub message: String,
}

impl SegmentPlanRule for SegmentPlanRequiredRegionRule {
    fn check(&self, item: &SegmentPlan) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPlanAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentPlanRule for SegmentPlanAttemptsRangeRule {
    fn check(&self, item: &SegmentPlan) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a segment plan.
#[derive(Debug, Clone)]
pub struct SegmentPlanValidator {
    pub rules: Vec<Rc<dyn SegmentPlanRule>>,
}

impl SegmentPlanValidator {
    pub fn validate(&self, item: &SegmentPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_plan_validator() -> SegmentPlanValidator {
    SegmentPlanValidator {
        rules: vec![
            Rc::new(SegmentPlanRegionLengthRule { max_length: 120 }) as Rc<dyn SegmentPlanRule>,
            Rc::new(SegmentPlanRequiredCodeRule {
                message: "segment plan needs an entry".to_string(),
            }),
            Rc::new(SegmentPlanRequiredCountryRule {
                message: "segment plan needs this field".to_string(),
            }),
            Rc::new(SegmentPlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SegmentPlanRequiredRegionRule {
                message: "segment plan needs an entry".to_string(),
            }),
            Rc::new(SegmentPlanAttemptsRangeRule { min: 16, max: 80 }),
        ],
    }
}
