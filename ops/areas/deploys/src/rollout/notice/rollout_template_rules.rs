use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout template must pass. Check returns its problems, or none.
pub trait RolloutTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RolloutTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTemplateLabelLengthRule {
    pub max_length: i64,
}

impl RolloutTemplateRule for RolloutTemplateLabelLengthRule {
    fn check(&self, item: &RolloutTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTemplateRequiredCodeRule {
    pub message: String,
}

impl RolloutTemplateRule for RolloutTemplateRequiredCodeRule {
    fn check(&self, item: &RolloutTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTemplateRequiredOwnerIdRule {
    pub message: String,
}

impl RolloutTemplateRule for RolloutTemplateRequiredOwnerIdRule {
    fn check(&self, item: &RolloutTemplate) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout template.
#[derive(Debug, Clone)]
pub struct RolloutTemplateValidator {
    pub rules: Vec<Rc<dyn RolloutTemplateRule>>,
}

impl RolloutTemplateValidator {
    pub fn validate(&self, item: &RolloutTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_template_validator() -> RolloutTemplateValidator {
    RolloutTemplateValidator {
        rules: vec![
            Rc::new(RolloutTemplateLabelLengthRule { max_length: 32 }) as Rc<dyn RolloutTemplateRule>,
            Rc::new(RolloutTemplateRequiredCodeRule {
                message: "rollout template needs a value".to_string(),
            }),
            Rc::new(RolloutTemplateRequiredOwnerIdRule {
                message: "rollout template needs an entry".to_string(),
            }),
        ],
    }
}
