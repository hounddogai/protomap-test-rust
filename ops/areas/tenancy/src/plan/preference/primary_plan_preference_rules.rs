use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary plan preference must pass. Check returns its problems, or none.
pub trait PrimaryPlanPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPlanPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanPreferenceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryPlanPreferenceRule for PrimaryPlanPreferenceAllowedCodeRule {
    fn check(&self, item: &PrimaryPlanPreference) -> Vec<String> {
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
pub struct PrimaryPlanPreferenceRequiredNameRule {
    pub message: String,
}

impl PrimaryPlanPreferenceRule for PrimaryPlanPreferenceRequiredNameRule {
    fn check(&self, item: &PrimaryPlanPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanPreferenceDescriptionLengthRule {
    pub max_length: i64,
}

impl PrimaryPlanPreferenceRule for PrimaryPlanPreferenceDescriptionLengthRule {
    fn check(&self, item: &PrimaryPlanPreference) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary plan preference.
#[derive(Debug, Clone)]
pub struct PrimaryPlanPreferenceValidator {
    pub rules: Vec<Rc<dyn PrimaryPlanPreferenceRule>>,
}

impl PrimaryPlanPreferenceValidator {
    pub fn validate(&self, item: &PrimaryPlanPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryPlanPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPlanPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_plan_preference_validator() -> PrimaryPlanPreferenceValidator {
    PrimaryPlanPreferenceValidator {
        rules: vec![
            Rc::new(PrimaryPlanPreferenceAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "batched".to_string()],
            }) as Rc<dyn PrimaryPlanPreferenceRule>,
            Rc::new(PrimaryPlanPreferenceRequiredNameRule {
                message: "primary plan preference needs an entry".to_string(),
            }),
            Rc::new(PrimaryPlanPreferenceDescriptionLengthRule { max_length: 255 }),
        ],
    }
}
