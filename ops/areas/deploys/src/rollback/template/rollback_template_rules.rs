use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback template must pass. Check returns its problems, or none.
pub trait RollbackTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RollbackTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTemplateTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackTemplateRule for RollbackTemplateTtlSecondsRangeRule {
    fn check(&self, item: &RollbackTemplate) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTemplateRequiredNameRule {
    pub message: String,
}

impl RollbackTemplateRule for RollbackTemplateRequiredNameRule {
    fn check(&self, item: &RollbackTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTemplateCodeLengthRule {
    pub max_length: i64,
}

impl RollbackTemplateRule for RollbackTemplateCodeLengthRule {
    fn check(&self, item: &RollbackTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackTemplateRequiredCipherRule {
    pub message: String,
}

impl RollbackTemplateRule for RollbackTemplateRequiredCipherRule {
    fn check(&self, item: &RollbackTemplate) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback template.
#[derive(Debug, Clone)]
pub struct RollbackTemplateValidator {
    pub rules: Vec<Rc<dyn RollbackTemplateRule>>,
}

impl RollbackTemplateValidator {
    pub fn validate(&self, item: &RollbackTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_template_validator() -> RollbackTemplateValidator {
    RollbackTemplateValidator {
        rules: vec![
            Rc::new(RollbackTemplateTtlSecondsRangeRule { min: 80, max: 3600 }) as Rc<dyn RollbackTemplateRule>,
            Rc::new(RollbackTemplateRequiredNameRule {
                message: "rollback template needs an entry".to_string(),
            }),
            Rc::new(RollbackTemplateCodeLengthRule { max_length: 86400 }),
            Rc::new(RollbackTemplateRequiredCipherRule {
                message: "rollback template needs this field".to_string(),
            }),
        ],
    }
}
