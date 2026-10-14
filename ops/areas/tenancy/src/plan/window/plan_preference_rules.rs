use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan preference must pass. Check returns its problems, or none.
pub trait PlanPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PlanPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanPreferenceCodePrefixRule {
    pub prefix: String,
}

impl PlanPreferenceRule for PlanPreferenceCodePrefixRule {
    fn check(&self, item: &PlanPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl PlanPreferenceRule for PlanPreferenceCodeLengthRule {
    fn check(&self, item: &PlanPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanPreferenceRequiredCodeRule {
    pub message: String,
}

impl PlanPreferenceRule for PlanPreferenceRequiredCodeRule {
    fn check(&self, item: &PlanPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan preference.
#[derive(Debug, Clone)]
pub struct PlanPreferenceValidator {
    pub rules: Vec<Rc<dyn PlanPreferenceRule>>,
}

impl PlanPreferenceValidator {
    pub fn validate(&self, item: &PlanPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_preference_validator() -> PlanPreferenceValidator {
    PlanPreferenceValidator {
        rules: vec![
            Rc::new(PlanPreferenceCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn PlanPreferenceRule>,
            Rc::new(PlanPreferenceCodeLengthRule { max_length: 86400 }),
            Rc::new(PlanPreferenceRequiredCodeRule {
                message: "plan preference needs this field".to_string(),
            }),
        ],
    }
}
