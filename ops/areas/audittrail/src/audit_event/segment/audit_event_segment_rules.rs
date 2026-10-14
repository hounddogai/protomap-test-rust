use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event segment must pass. Check returns its problems, or none.
pub trait AuditEventSegmentRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSegmentCodePrefixRule {
    pub prefix: String,
}

impl AuditEventSegmentRule for AuditEventSegmentCodePrefixRule {
    fn check(&self, item: &AuditEventSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSegmentRequiredChecksumRule {
    pub message: String,
}

impl AuditEventSegmentRule for AuditEventSegmentRequiredChecksumRule {
    fn check(&self, item: &AuditEventSegment) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSegmentRequiredTitleRule {
    pub message: String,
}

impl AuditEventSegmentRule for AuditEventSegmentRequiredTitleRule {
    fn check(&self, item: &AuditEventSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSegmentRequiredCodeRule {
    pub message: String,
}

impl AuditEventSegmentRule for AuditEventSegmentRequiredCodeRule {
    fn check(&self, item: &AuditEventSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSegmentSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventSegmentRule for AuditEventSegmentSequenceRangeRule {
    fn check(&self, item: &AuditEventSegment) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event segment.
#[derive(Debug, Clone)]
pub struct AuditEventSegmentValidator {
    pub rules: Vec<Rc<dyn AuditEventSegmentRule>>,
}

impl AuditEventSegmentValidator {
    pub fn validate(&self, item: &AuditEventSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_segment_validator() -> AuditEventSegmentValidator {
    AuditEventSegmentValidator {
        rules: vec![
            Rc::new(AuditEventSegmentCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn AuditEventSegmentRule>,
            Rc::new(AuditEventSegmentRequiredChecksumRule {
                message: "audit event segment needs a value".to_string(),
            }),
            Rc::new(AuditEventSegmentRequiredTitleRule {
                message: "audit event segment needs an entry".to_string(),
            }),
            Rc::new(AuditEventSegmentRequiredCodeRule {
                message: "audit event segment needs a value".to_string(),
            }),
            Rc::new(AuditEventSegmentSequenceRangeRule { min: 1000, max: 1000 }),
        ],
    }
}
