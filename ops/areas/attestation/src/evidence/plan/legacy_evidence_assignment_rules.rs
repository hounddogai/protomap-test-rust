use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy evidence assignment must pass. Check returns its problems, or none.
pub trait LegacyEvidenceAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyEvidenceAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceAssignmentRequiredCodeRule {
    pub message: String,
}

impl LegacyEvidenceAssignmentRule for LegacyEvidenceAssignmentRequiredCodeRule {
    fn check(&self, item: &LegacyEvidenceAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceAssignmentCodePrefixRule {
    pub prefix: String,
}

impl LegacyEvidenceAssignmentRule for LegacyEvidenceAssignmentCodePrefixRule {
    fn check(&self, item: &LegacyEvidenceAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceAssignmentUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyEvidenceAssignmentRule for LegacyEvidenceAssignmentUpdatedAtRangeRule {
    fn check(&self, item: &LegacyEvidenceAssignment) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceAssignmentNameLengthRule {
    pub max_length: i64,
}

impl LegacyEvidenceAssignmentRule for LegacyEvidenceAssignmentNameLengthRule {
    fn check(&self, item: &LegacyEvidenceAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceAssignmentRequiredSummaryRule {
    pub message: String,
}

impl LegacyEvidenceAssignmentRule for LegacyEvidenceAssignmentRequiredSummaryRule {
    fn check(&self, item: &LegacyEvidenceAssignment) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy evidence assignment.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceAssignmentValidator {
    pub rules: Vec<Rc<dyn LegacyEvidenceAssignmentRule>>,
}

impl LegacyEvidenceAssignmentValidator {
    pub fn validate(&self, item: &LegacyEvidenceAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyEvidenceAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyEvidenceAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_evidence_assignment_validator() -> LegacyEvidenceAssignmentValidator {
    LegacyEvidenceAssignmentValidator {
        rules: vec![
            Rc::new(LegacyEvidenceAssignmentRequiredCodeRule {
                message: "legacy evidence assignment needs an entry".to_string(),
            }) as Rc<dyn LegacyEvidenceAssignmentRule>,
            Rc::new(LegacyEvidenceAssignmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LegacyEvidenceAssignmentUpdatedAtRangeRule { min: 80, max: 255 }),
            Rc::new(LegacyEvidenceAssignmentNameLengthRule { max_length: 120 }),
            Rc::new(LegacyEvidenceAssignmentRequiredSummaryRule {
                message: "legacy evidence assignment needs an entry".to_string(),
            }),
        ],
    }
}
