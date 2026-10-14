use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive rule assignment must pass. Check returns its problems, or none.
pub trait AdaptiveRuleAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRuleAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveRuleAssignmentRule for AdaptiveRuleAssignmentCodeLengthRule {
    fn check(&self, item: &AdaptiveRuleAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleAssignmentDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRuleAssignmentRule for AdaptiveRuleAssignmentDiscountRangeRule {
    fn check(&self, item: &AdaptiveRuleAssignment) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleAssignmentRequiredLabelRule {
    pub message: String,
}

impl AdaptiveRuleAssignmentRule for AdaptiveRuleAssignmentRequiredLabelRule {
    fn check(&self, item: &AdaptiveRuleAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleAssignmentPinnedGuardRule {
    pub limit: i64,
}

impl AdaptiveRuleAssignmentRule for AdaptiveRuleAssignmentPinnedGuardRule {
    fn check(&self, item: &AdaptiveRuleAssignment) -> Vec<String> {
        if item.pinned && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while pinned",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive rule assignment.
#[derive(Debug, Clone)]
pub struct AdaptiveRuleAssignmentValidator {
    pub rules: Vec<Rc<dyn AdaptiveRuleAssignmentRule>>,
}

impl AdaptiveRuleAssignmentValidator {
    pub fn validate(&self, item: &AdaptiveRuleAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRuleAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRuleAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_rule_assignment_validator() -> AdaptiveRuleAssignmentValidator {
    AdaptiveRuleAssignmentValidator {
        rules: vec![
            Rc::new(AdaptiveRuleAssignmentCodeLengthRule { max_length: 64 }) as Rc<dyn AdaptiveRuleAssignmentRule>,
            Rc::new(AdaptiveRuleAssignmentDiscountRangeRule { min: 1.7, max: 2379.9 }),
            Rc::new(AdaptiveRuleAssignmentRequiredLabelRule {
                message: "adaptive rule assignment needs a value".to_string(),
            }),
            Rc::new(AdaptiveRuleAssignmentPinnedGuardRule { limit: 1000 }),
        ],
    }
}
