use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan audit must pass. Check returns its problems, or none.
pub trait PlanAuditRule: std::fmt::Debug {
    fn check(&self, item: &PlanAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAuditQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanAuditRule for PlanAuditQuotaUnitsRangeRule {
    fn check(&self, item: &PlanAudit) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAuditCodePrefixRule {
    pub prefix: String,
}

impl PlanAuditRule for PlanAuditCodePrefixRule {
    fn check(&self, item: &PlanAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAuditThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanAuditRule for PlanAuditThresholdRangeRule {
    fn check(&self, item: &PlanAudit) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAuditCodeLengthRule {
    pub max_length: i64,
}

impl PlanAuditRule for PlanAuditCodeLengthRule {
    fn check(&self, item: &PlanAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAuditAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PlanAuditRule for PlanAuditAllowedCodeRule {
    fn check(&self, item: &PlanAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAuditRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PlanAuditRule for PlanAuditRatioRangeRule {
    fn check(&self, item: &PlanAudit) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan audit.
#[derive(Debug, Clone)]
pub struct PlanAuditValidator {
    pub rules: Vec<Rc<dyn PlanAuditRule>>,
}

impl PlanAuditValidator {
    pub fn validate(&self, item: &PlanAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_audit_validator() -> PlanAuditValidator {
    PlanAuditValidator {
        rules: vec![
            Rc::new(PlanAuditQuotaUnitsRangeRule { min: 32, max: 3600 }) as Rc<dyn PlanAuditRule>,
            Rc::new(PlanAuditCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PlanAuditThresholdRangeRule { min: 6.6, max: 1752.4 }),
            Rc::new(PlanAuditCodeLengthRule { max_length: 16 }),
            Rc::new(PlanAuditAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PlanAuditRatioRangeRule { min: 7.9, max: 2698.2 }),
        ],
    }
}
