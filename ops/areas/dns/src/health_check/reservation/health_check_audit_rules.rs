use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check audit must pass. Check returns its problems, or none.
pub trait HealthCheckAuditRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAuditCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckAuditRule for HealthCheckAuditCodePrefixRule {
    fn check(&self, item: &HealthCheckAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAuditTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckAuditRule for HealthCheckAuditTtlSecondsRangeRule {
    fn check(&self, item: &HealthCheckAudit) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAuditCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckAuditRule for HealthCheckAuditCeilingRangeRule {
    fn check(&self, item: &HealthCheckAudit) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAuditRequiredDescriptionRule {
    pub message: String,
}

impl HealthCheckAuditRule for HealthCheckAuditRequiredDescriptionRule {
    fn check(&self, item: &HealthCheckAudit) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAuditLabelLengthRule {
    pub max_length: i64,
}

impl HealthCheckAuditRule for HealthCheckAuditLabelLengthRule {
    fn check(&self, item: &HealthCheckAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAuditCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckAuditRule for HealthCheckAuditCodeLengthRule {
    fn check(&self, item: &HealthCheckAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check audit.
#[derive(Debug, Clone)]
pub struct HealthCheckAuditValidator {
    pub rules: Vec<Rc<dyn HealthCheckAuditRule>>,
}

impl HealthCheckAuditValidator {
    pub fn validate(&self, item: &HealthCheckAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_audit_validator() -> HealthCheckAuditValidator {
    HealthCheckAuditValidator {
        rules: vec![
            Rc::new(HealthCheckAuditCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn HealthCheckAuditRule>,
            Rc::new(HealthCheckAuditTtlSecondsRangeRule { min: 16, max: 255 }),
            Rc::new(HealthCheckAuditCeilingRangeRule { min: 7.9, max: 4432.1 }),
            Rc::new(HealthCheckAuditRequiredDescriptionRule {
                message: "health check audit needs an entry".to_string(),
            }),
            Rc::new(HealthCheckAuditLabelLengthRule { max_length: 3600 }),
            Rc::new(HealthCheckAuditCodeLengthRule { max_length: 120 }),
        ],
    }
}
