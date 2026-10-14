use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation assignment must pass. Check returns its problems, or none.
pub trait ReservationAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ReservationAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl ReservationAssignmentRule for ReservationAssignmentCodeLengthRule {
    fn check(&self, item: &ReservationAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAssignmentRegionLengthRule {
    pub max_length: i64,
}

impl ReservationAssignmentRule for ReservationAssignmentRegionLengthRule {
    fn check(&self, item: &ReservationAssignment) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAssignmentRequiredLabelRule {
    pub message: String,
}

impl ReservationAssignmentRule for ReservationAssignmentRequiredLabelRule {
    fn check(&self, item: &ReservationAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAssignmentNameLengthRule {
    pub max_length: i64,
}

impl ReservationAssignmentRule for ReservationAssignmentNameLengthRule {
    fn check(&self, item: &ReservationAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAssignmentLabelLengthRule {
    pub max_length: i64,
}

impl ReservationAssignmentRule for ReservationAssignmentLabelLengthRule {
    fn check(&self, item: &ReservationAssignment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation assignment.
#[derive(Debug, Clone)]
pub struct ReservationAssignmentValidator {
    pub rules: Vec<Rc<dyn ReservationAssignmentRule>>,
}

impl ReservationAssignmentValidator {
    pub fn validate(&self, item: &ReservationAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_assignment_validator() -> ReservationAssignmentValidator {
    ReservationAssignmentValidator {
        rules: vec![
            Rc::new(ReservationAssignmentCodeLengthRule { max_length: 64 }) as Rc<dyn ReservationAssignmentRule>,
            Rc::new(ReservationAssignmentRegionLengthRule { max_length: 32 }),
            Rc::new(ReservationAssignmentRequiredLabelRule {
                message: "reservation assignment needs a value".to_string(),
            }),
            Rc::new(ReservationAssignmentNameLengthRule { max_length: 86400 }),
            Rc::new(ReservationAssignmentLabelLengthRule { max_length: 3600 }),
        ],
    }
}
