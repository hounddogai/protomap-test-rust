use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence assignment must pass. Check returns its problems, or none.
pub trait EvidenceAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAssignmentAudienceLengthRule {
    pub max_length: i64,
}

impl EvidenceAssignmentRule for EvidenceAssignmentAudienceLengthRule {
    fn check(&self, item: &EvidenceAssignment) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAssignmentExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceAssignmentRule for EvidenceAssignmentExpiresAtRangeRule {
    fn check(&self, item: &EvidenceAssignment) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAssignmentRequiredUnitRule {
    pub message: String,
}

impl EvidenceAssignmentRule for EvidenceAssignmentRequiredUnitRule {
    fn check(&self, item: &EvidenceAssignment) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAssignmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceAssignmentRule for EvidenceAssignmentCostRangeRule {
    fn check(&self, item: &EvidenceAssignment) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a evidence assignment.
#[derive(Debug, Clone)]
pub struct EvidenceAssignmentValidator {
    pub rules: Vec<Rc<dyn EvidenceAssignmentRule>>,
}

impl EvidenceAssignmentValidator {
    pub fn validate(&self, item: &EvidenceAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_assignment_validator() -> EvidenceAssignmentValidator {
    EvidenceAssignmentValidator {
        rules: vec![
            Rc::new(EvidenceAssignmentAudienceLengthRule { max_length: 120 }) as Rc<dyn EvidenceAssignmentRule>,
            Rc::new(EvidenceAssignmentExpiresAtRangeRule { min: 32, max: 1000 }),
            Rc::new(EvidenceAssignmentRequiredUnitRule {
                message: "evidence assignment needs an entry".to_string(),
            }),
            Rc::new(EvidenceAssignmentCostRangeRule { min: 7.5, max: 4278.9 }),
        ],
    }
}
