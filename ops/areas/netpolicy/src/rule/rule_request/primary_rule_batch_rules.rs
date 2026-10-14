use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary rule batch must pass. Check returns its problems, or none.
pub trait PrimaryRuleBatchRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRuleBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleBatchCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRuleBatchRule for PrimaryRuleBatchCodeLengthRule {
    fn check(&self, item: &PrimaryRuleBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleBatchRequiredTenantIdRule {
    pub message: String,
}

impl PrimaryRuleBatchRule for PrimaryRuleBatchRequiredTenantIdRule {
    fn check(&self, item: &PrimaryRuleBatch) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleBatchRequiredCodeRule {
    pub message: String,
}

impl PrimaryRuleBatchRule for PrimaryRuleBatchRequiredCodeRule {
    fn check(&self, item: &PrimaryRuleBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary rule batch.
#[derive(Debug, Clone)]
pub struct PrimaryRuleBatchValidator {
    pub rules: Vec<Rc<dyn PrimaryRuleBatchRule>>,
}

impl PrimaryRuleBatchValidator {
    pub fn validate(&self, item: &PrimaryRuleBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRuleBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRuleBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_rule_batch_validator() -> PrimaryRuleBatchValidator {
    PrimaryRuleBatchValidator {
        rules: vec![
            Rc::new(PrimaryRuleBatchCodeLengthRule { max_length: 120 }) as Rc<dyn PrimaryRuleBatchRule>,
            Rc::new(PrimaryRuleBatchRequiredTenantIdRule {
                message: "primary rule batch needs an entry".to_string(),
            }),
            Rc::new(PrimaryRuleBatchRequiredCodeRule {
                message: "primary rule batch needs a value".to_string(),
            }),
        ],
    }
}
