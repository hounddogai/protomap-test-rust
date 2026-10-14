use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive timeline plan must pass. Check returns its problems, or none.
pub trait AdaptiveTimelinePlanRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTimelinePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelinePlanAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AdaptiveTimelinePlanRule for AdaptiveTimelinePlanAllowedLabelRule {
    fn check(&self, item: &AdaptiveTimelinePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelinePlanRequiredReviewerRule {
    pub message: String,
}

impl AdaptiveTimelinePlanRule for AdaptiveTimelinePlanRequiredReviewerRule {
    fn check(&self, item: &AdaptiveTimelinePlan) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelinePlanCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveTimelinePlanRule for AdaptiveTimelinePlanCodePrefixRule {
    fn check(&self, item: &AdaptiveTimelinePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive timeline plan.
#[derive(Debug, Clone)]
pub struct AdaptiveTimelinePlanValidator {
    pub rules: Vec<Rc<dyn AdaptiveTimelinePlanRule>>,
}

impl AdaptiveTimelinePlanValidator {
    pub fn validate(&self, item: &AdaptiveTimelinePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveTimelinePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTimelinePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_timeline_plan_validator() -> AdaptiveTimelinePlanValidator {
    AdaptiveTimelinePlanValidator {
        rules: vec![
            Rc::new(AdaptiveTimelinePlanAllowedLabelRule { allowed: Vec::new() }) as Rc<dyn AdaptiveTimelinePlanRule>,
            Rc::new(AdaptiveTimelinePlanRequiredReviewerRule {
                message: "adaptive timeline plan needs this field".to_string(),
            }),
            Rc::new(AdaptiveTimelinePlanCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
