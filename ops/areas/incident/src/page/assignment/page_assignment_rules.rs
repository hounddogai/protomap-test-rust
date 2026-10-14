use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page assignment must pass. Check returns its problems, or none.
pub trait PageAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &PageAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAssignmentRequiredCodeRule {
    pub message: String,
}

impl PageAssignmentRule for PageAssignmentRequiredCodeRule {
    fn check(&self, item: &PageAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAssignmentRequiredNameRule {
    pub message: String,
}

impl PageAssignmentRule for PageAssignmentRequiredNameRule {
    fn check(&self, item: &PageAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAssignmentRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageAssignmentRule for PageAssignmentRevisionRangeRule {
    fn check(&self, item: &PageAssignment) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAssignmentRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageAssignmentRule for PageAssignmentRotationDaysRangeRule {
    fn check(&self, item: &PageAssignment) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a page assignment.
#[derive(Debug, Clone)]
pub struct PageAssignmentValidator {
    pub rules: Vec<Rc<dyn PageAssignmentRule>>,
}

impl PageAssignmentValidator {
    pub fn validate(&self, item: &PageAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_assignment_validator() -> PageAssignmentValidator {
    PageAssignmentValidator {
        rules: vec![
            Rc::new(PageAssignmentRequiredCodeRule {
                message: "page assignment needs this field".to_string(),
            }) as Rc<dyn PageAssignmentRule>,
            Rc::new(PageAssignmentRequiredNameRule {
                message: "page assignment needs a value".to_string(),
            }),
            Rc::new(PageAssignmentRevisionRangeRule { min: 16, max: 3600 }),
            Rc::new(PageAssignmentRotationDaysRangeRule { min: 120, max: 86400 }),
        ],
    }
}
