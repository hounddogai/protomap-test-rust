use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence assignment must pass. Check returns its problems, or none.
pub trait SilenceAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &SilenceAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAssignmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceAssignmentRule for SilenceAssignmentRateRangeRule {
    fn check(&self, item: &SilenceAssignment) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl SilenceAssignmentRule for SilenceAssignmentCodeLengthRule {
    fn check(&self, item: &SilenceAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAssignmentNameLengthRule {
    pub max_length: i64,
}

impl SilenceAssignmentRule for SilenceAssignmentNameLengthRule {
    fn check(&self, item: &SilenceAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence assignment.
#[derive(Debug, Clone)]
pub struct SilenceAssignmentValidator {
    pub rules: Vec<Rc<dyn SilenceAssignmentRule>>,
}

impl SilenceAssignmentValidator {
    pub fn validate(&self, item: &SilenceAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_assignment_validator() -> SilenceAssignmentValidator {
    SilenceAssignmentValidator {
        rules: vec![
            Rc::new(SilenceAssignmentRateRangeRule { min: 4.8, max: 2560.6 }) as Rc<dyn SilenceAssignmentRule>,
            Rc::new(SilenceAssignmentCodeLengthRule { max_length: 1000 }),
            Rc::new(SilenceAssignmentNameLengthRule { max_length: 255 }),
        ],
    }
}
