use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional runbook plan must pass. Check returns its problems, or none.
pub trait RegionalRunbookPlanRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRunbookPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookPlanRequiredOwnerIdRule {
    pub message: String,
}

impl RegionalRunbookPlanRule for RegionalRunbookPlanRequiredOwnerIdRule {
    fn check(&self, item: &RegionalRunbookPlan) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookPlanRequiredLabelRule {
    pub message: String,
}

impl RegionalRunbookPlanRule for RegionalRunbookPlanRequiredLabelRule {
    fn check(&self, item: &RegionalRunbookPlan) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookPlanVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRunbookPlanRule for RegionalRunbookPlanVersionRangeRule {
    fn check(&self, item: &RegionalRunbookPlan) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookPlanExternalRefLengthRule {
    pub max_length: i64,
}

impl RegionalRunbookPlanRule for RegionalRunbookPlanExternalRefLengthRule {
    fn check(&self, item: &RegionalRunbookPlan) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional runbook plan.
#[derive(Debug, Clone)]
pub struct RegionalRunbookPlanValidator {
    pub rules: Vec<Rc<dyn RegionalRunbookPlanRule>>,
}

impl RegionalRunbookPlanValidator {
    pub fn validate(&self, item: &RegionalRunbookPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalRunbookPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRunbookPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_runbook_plan_validator() -> RegionalRunbookPlanValidator {
    RegionalRunbookPlanValidator {
        rules: vec![
            Rc::new(RegionalRunbookPlanRequiredOwnerIdRule {
                message: "regional runbook plan needs an entry".to_string(),
            }) as Rc<dyn RegionalRunbookPlanRule>,
            Rc::new(RegionalRunbookPlanRequiredLabelRule {
                message: "regional runbook plan needs this field".to_string(),
            }),
            Rc::new(RegionalRunbookPlanVersionRangeRule { min: 120, max: 255 }),
            Rc::new(RegionalRunbookPlanExternalRefLengthRule { max_length: 1000 }),
        ],
    }
}
