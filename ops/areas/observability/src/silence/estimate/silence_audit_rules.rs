use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence audit must pass. Check returns its problems, or none.
pub trait SilenceAuditRule: std::fmt::Debug {
    fn check(&self, item: &SilenceAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAuditPrincipalLengthRule {
    pub max_length: i64,
}

impl SilenceAuditRule for SilenceAuditPrincipalLengthRule {
    fn check(&self, item: &SilenceAudit) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAuditRequiredChecksumRule {
    pub message: String,
}

impl SilenceAuditRule for SilenceAuditRequiredChecksumRule {
    fn check(&self, item: &SilenceAudit) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAuditCodeLengthRule {
    pub max_length: i64,
}

impl SilenceAuditRule for SilenceAuditCodeLengthRule {
    fn check(&self, item: &SilenceAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAuditThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceAuditRule for SilenceAuditThresholdRangeRule {
    fn check(&self, item: &SilenceAudit) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAuditRequiredNameRule {
    pub message: String,
}

impl SilenceAuditRule for SilenceAuditRequiredNameRule {
    fn check(&self, item: &SilenceAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence audit.
#[derive(Debug, Clone)]
pub struct SilenceAuditValidator {
    pub rules: Vec<Rc<dyn SilenceAuditRule>>,
}

impl SilenceAuditValidator {
    pub fn validate(&self, item: &SilenceAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_audit_validator() -> SilenceAuditValidator {
    SilenceAuditValidator {
        rules: vec![
            Rc::new(SilenceAuditPrincipalLengthRule { max_length: 86400 }) as Rc<dyn SilenceAuditRule>,
            Rc::new(SilenceAuditRequiredChecksumRule {
                message: "silence audit needs this field".to_string(),
            }),
            Rc::new(SilenceAuditCodeLengthRule { max_length: 80 }),
            Rc::new(SilenceAuditThresholdRangeRule { min: 0.2, max: 1951.9 }),
            Rc::new(SilenceAuditRequiredNameRule {
                message: "silence audit needs a value".to_string(),
            }),
        ],
    }
}
