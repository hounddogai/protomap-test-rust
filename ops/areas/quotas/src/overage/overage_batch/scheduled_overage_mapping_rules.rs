use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled overage mapping must pass. Check returns its problems, or none.
pub trait ScheduledOverageMappingRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledOverageMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageMappingRequiredTitleRule {
    pub message: String,
}

impl ScheduledOverageMappingRule for ScheduledOverageMappingRequiredTitleRule {
    fn check(&self, item: &ScheduledOverageMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageMappingAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledOverageMappingRule for ScheduledOverageMappingAllowedCodeRule {
    fn check(&self, item: &ScheduledOverageMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageMappingCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledOverageMappingRule for ScheduledOverageMappingCodeLengthRule {
    fn check(&self, item: &ScheduledOverageMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled overage mapping.
#[derive(Debug, Clone)]
pub struct ScheduledOverageMappingValidator {
    pub rules: Vec<Rc<dyn ScheduledOverageMappingRule>>,
}

impl ScheduledOverageMappingValidator {
    pub fn validate(&self, item: &ScheduledOverageMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledOverageMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledOverageMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_overage_mapping_validator() -> ScheduledOverageMappingValidator {
    ScheduledOverageMappingValidator {
        rules: vec![
            Rc::new(ScheduledOverageMappingRequiredTitleRule {
                message: "scheduled overage mapping needs this field".to_string(),
            }) as Rc<dyn ScheduledOverageMappingRule>,
            Rc::new(ScheduledOverageMappingAllowedCodeRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(ScheduledOverageMappingCodeLengthRule { max_length: 64 }),
        ],
    }
}
