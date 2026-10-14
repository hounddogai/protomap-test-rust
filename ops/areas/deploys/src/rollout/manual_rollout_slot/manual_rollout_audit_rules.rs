use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual rollout audit must pass. Check returns its problems, or none.
pub trait ManualRolloutAuditRule: std::fmt::Debug {
    fn check(&self, item: &ManualRolloutAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRolloutAuditRequiredNameRule {
    pub message: String,
}

impl ManualRolloutAuditRule for ManualRolloutAuditRequiredNameRule {
    fn check(&self, item: &ManualRolloutAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRolloutAuditPrincipalLengthRule {
    pub max_length: i64,
}

impl ManualRolloutAuditRule for ManualRolloutAuditPrincipalLengthRule {
    fn check(&self, item: &ManualRolloutAudit) -> Vec<String> {
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
pub struct ManualRolloutAuditQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualRolloutAuditRule for ManualRolloutAuditQuantityRangeRule {
    fn check(&self, item: &ManualRolloutAudit) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual rollout audit.
#[derive(Debug, Clone)]
pub struct ManualRolloutAuditValidator {
    pub rules: Vec<Rc<dyn ManualRolloutAuditRule>>,
}

impl ManualRolloutAuditValidator {
    pub fn validate(&self, item: &ManualRolloutAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRolloutAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRolloutAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_rollout_audit_validator() -> ManualRolloutAuditValidator {
    ManualRolloutAuditValidator {
        rules: vec![
            Rc::new(ManualRolloutAuditRequiredNameRule {
                message: "manual rollout audit needs a value".to_string(),
            }) as Rc<dyn ManualRolloutAuditRule>,
            Rc::new(ManualRolloutAuditPrincipalLengthRule { max_length: 1000 }),
            Rc::new(ManualRolloutAuditQuantityRangeRule { min: 80, max: 1000 }),
        ],
    }
}
