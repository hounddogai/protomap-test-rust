use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule assignment must pass. Check returns its problems, or none.
pub trait RuleAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &RuleAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAssignmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleAssignmentRule for RuleAssignmentCostRangeRule {
    fn check(&self, item: &RuleAssignment) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAssignmentExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleAssignmentRule for RuleAssignmentExpiresAtRangeRule {
    fn check(&self, item: &RuleAssignment) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAssignmentLocaleLengthRule {
    pub max_length: i64,
}

impl RuleAssignmentRule for RuleAssignmentLocaleLengthRule {
    fn check(&self, item: &RuleAssignment) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAssignmentRequiredCodeRule {
    pub message: String,
}

impl RuleAssignmentRule for RuleAssignmentRequiredCodeRule {
    fn check(&self, item: &RuleAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAssignmentCountryLengthRule {
    pub max_length: i64,
}

impl RuleAssignmentRule for RuleAssignmentCountryLengthRule {
    fn check(&self, item: &RuleAssignment) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule assignment.
#[derive(Debug, Clone)]
pub struct RuleAssignmentValidator {
    pub rules: Vec<Rc<dyn RuleAssignmentRule>>,
}

impl RuleAssignmentValidator {
    pub fn validate(&self, item: &RuleAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_assignment_validator() -> RuleAssignmentValidator {
    RuleAssignmentValidator {
        rules: vec![
            Rc::new(RuleAssignmentCostRangeRule { min: 7.2, max: 1903.6 }) as Rc<dyn RuleAssignmentRule>,
            Rc::new(RuleAssignmentExpiresAtRangeRule { min: 120, max: 3600 }),
            Rc::new(RuleAssignmentLocaleLengthRule { max_length: 80 }),
            Rc::new(RuleAssignmentRequiredCodeRule {
                message: "rule assignment needs a value".to_string(),
            }),
            Rc::new(RuleAssignmentCountryLengthRule { max_length: 32 }),
        ],
    }
}
