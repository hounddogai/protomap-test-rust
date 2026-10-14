use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook milestone must pass. Check returns its problems, or none.
pub trait RunbookMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &RunbookMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl RunbookMilestoneRule for RunbookMilestoneCodeLengthRule {
    fn check(&self, item: &RunbookMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMilestoneRequiredAudienceRule {
    pub message: String,
}

impl RunbookMilestoneRule for RunbookMilestoneRequiredAudienceRule {
    fn check(&self, item: &RunbookMilestone) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMilestoneCodePrefixRule {
    pub prefix: String,
}

impl RunbookMilestoneRule for RunbookMilestoneCodePrefixRule {
    fn check(&self, item: &RunbookMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook milestone.
#[derive(Debug, Clone)]
pub struct RunbookMilestoneValidator {
    pub rules: Vec<Rc<dyn RunbookMilestoneRule>>,
}

impl RunbookMilestoneValidator {
    pub fn validate(&self, item: &RunbookMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_milestone_validator() -> RunbookMilestoneValidator {
    RunbookMilestoneValidator {
        rules: vec![
            Rc::new(RunbookMilestoneCodeLengthRule { max_length: 1000 }) as Rc<dyn RunbookMilestoneRule>,
            Rc::new(RunbookMilestoneRequiredAudienceRule {
                message: "runbook milestone needs a value".to_string(),
            }),
            Rc::new(RunbookMilestoneCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
