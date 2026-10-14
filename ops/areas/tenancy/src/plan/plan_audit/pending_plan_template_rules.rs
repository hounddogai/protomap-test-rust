use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending plan template must pass. Check returns its problems, or none.
pub trait PendingPlanTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PendingPlanTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanTemplateCodePrefixRule {
    pub prefix: String,
}

impl PendingPlanTemplateRule for PendingPlanTemplateCodePrefixRule {
    fn check(&self, item: &PendingPlanTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanTemplateRequiredCodeRule {
    pub message: String,
}

impl PendingPlanTemplateRule for PendingPlanTemplateRequiredCodeRule {
    fn check(&self, item: &PendingPlanTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanTemplateNameLengthRule {
    pub max_length: i64,
}

impl PendingPlanTemplateRule for PendingPlanTemplateNameLengthRule {
    fn check(&self, item: &PendingPlanTemplate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending plan template.
#[derive(Debug, Clone)]
pub struct PendingPlanTemplateValidator {
    pub rules: Vec<Rc<dyn PendingPlanTemplateRule>>,
}

impl PendingPlanTemplateValidator {
    pub fn validate(&self, item: &PendingPlanTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPlanTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPlanTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_plan_template_validator() -> PendingPlanTemplateValidator {
    PendingPlanTemplateValidator {
        rules: vec![
            Rc::new(PendingPlanTemplateCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn PendingPlanTemplateRule>,
            Rc::new(PendingPlanTemplateRequiredCodeRule {
                message: "pending plan template needs an entry".to_string(),
            }),
            Rc::new(PendingPlanTemplateNameLengthRule { max_length: 80 }),
        ],
    }
}
