use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone assignment must pass. Check returns its problems, or none.
pub trait ZoneAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ZoneAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAssignmentSummaryLengthRule {
    pub max_length: i64,
}

impl ZoneAssignmentRule for ZoneAssignmentSummaryLengthRule {
    fn check(&self, item: &ZoneAssignment) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAssignmentRequiredIssuerRule {
    pub message: String,
}

impl ZoneAssignmentRule for ZoneAssignmentRequiredIssuerRule {
    fn check(&self, item: &ZoneAssignment) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAssignmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ZoneAssignmentRule for ZoneAssignmentAllowedTitleRule {
    fn check(&self, item: &ZoneAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone assignment.
#[derive(Debug, Clone)]
pub struct ZoneAssignmentValidator {
    pub rules: Vec<Rc<dyn ZoneAssignmentRule>>,
}

impl ZoneAssignmentValidator {
    pub fn validate(&self, item: &ZoneAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ZoneAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_assignment_validator() -> ZoneAssignmentValidator {
    ZoneAssignmentValidator {
        rules: vec![
            Rc::new(ZoneAssignmentSummaryLengthRule { max_length: 120 }) as Rc<dyn ZoneAssignmentRule>,
            Rc::new(ZoneAssignmentRequiredIssuerRule {
                message: "zone assignment needs an entry".to_string(),
            }),
            Rc::new(ZoneAssignmentAllowedTitleRule {
                allowed: vec!["rolling".to_string()],
            }),
        ],
    }
}
