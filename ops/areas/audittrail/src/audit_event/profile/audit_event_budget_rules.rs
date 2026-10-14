use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event budget must pass. Check returns its problems, or none.
pub trait AuditEventBudgetRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBudgetKeyIdLengthRule {
    pub max_length: i64,
}

impl AuditEventBudgetRule for AuditEventBudgetKeyIdLengthRule {
    fn check(&self, item: &AuditEventBudget) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBudgetTenantIdLengthRule {
    pub max_length: i64,
}

impl AuditEventBudgetRule for AuditEventBudgetTenantIdLengthRule {
    fn check(&self, item: &AuditEventBudget) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBudgetRequiredLabelRule {
    pub message: String,
}

impl AuditEventBudgetRule for AuditEventBudgetRequiredLabelRule {
    fn check(&self, item: &AuditEventBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBudgetRequiredCodeRule {
    pub message: String,
}

impl AuditEventBudgetRule for AuditEventBudgetRequiredCodeRule {
    fn check(&self, item: &AuditEventBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBudgetStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventBudgetRule for AuditEventBudgetStartsAtRangeRule {
    fn check(&self, item: &AuditEventBudget) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a audit event budget.
#[derive(Debug, Clone)]
pub struct AuditEventBudgetValidator {
    pub rules: Vec<Rc<dyn AuditEventBudgetRule>>,
}

impl AuditEventBudgetValidator {
    pub fn validate(&self, item: &AuditEventBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_budget_validator() -> AuditEventBudgetValidator {
    AuditEventBudgetValidator {
        rules: vec![
            Rc::new(AuditEventBudgetKeyIdLengthRule { max_length: 16 }) as Rc<dyn AuditEventBudgetRule>,
            Rc::new(AuditEventBudgetTenantIdLengthRule { max_length: 120 }),
            Rc::new(AuditEventBudgetRequiredLabelRule {
                message: "audit event budget needs a value".to_string(),
            }),
            Rc::new(AuditEventBudgetRequiredCodeRule {
                message: "audit event budget needs this field".to_string(),
            }),
            Rc::new(AuditEventBudgetStartsAtRangeRule { min: 255, max: 1000 }),
        ],
    }
}
