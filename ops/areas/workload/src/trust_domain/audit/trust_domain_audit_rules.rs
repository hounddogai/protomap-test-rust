use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain audit must pass. Check returns its problems, or none.
pub trait TrustDomainAuditRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAuditTitleLengthRule {
    pub max_length: i64,
}

impl TrustDomainAuditRule for TrustDomainAuditTitleLengthRule {
    fn check(&self, item: &TrustDomainAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAuditFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainAuditRule for TrustDomainAuditFloorValueRangeRule {
    fn check(&self, item: &TrustDomainAudit) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAuditRequiredCodeRule {
    pub message: String,
}

impl TrustDomainAuditRule for TrustDomainAuditRequiredCodeRule {
    fn check(&self, item: &TrustDomainAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAuditRequiredSummaryRule {
    pub message: String,
}

impl TrustDomainAuditRule for TrustDomainAuditRequiredSummaryRule {
    fn check(&self, item: &TrustDomainAudit) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain audit.
#[derive(Debug, Clone)]
pub struct TrustDomainAuditValidator {
    pub rules: Vec<Rc<dyn TrustDomainAuditRule>>,
}

impl TrustDomainAuditValidator {
    pub fn validate(&self, item: &TrustDomainAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_audit_validator() -> TrustDomainAuditValidator {
    TrustDomainAuditValidator {
        rules: vec![
            Rc::new(TrustDomainAuditTitleLengthRule { max_length: 255 }) as Rc<dyn TrustDomainAuditRule>,
            Rc::new(TrustDomainAuditFloorValueRangeRule { min: 7.0, max: 1274.7 }),
            Rc::new(TrustDomainAuditRequiredCodeRule {
                message: "trust domain audit needs this field".to_string(),
            }),
            Rc::new(TrustDomainAuditRequiredSummaryRule {
                message: "trust domain audit needs this field".to_string(),
            }),
        ],
    }
}
