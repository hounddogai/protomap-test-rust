use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier preference must pass. Check returns its problems, or none.
pub trait VerifierPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &VerifierPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPreferenceFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierPreferenceRule for VerifierPreferenceFloorValueRangeRule {
    fn check(&self, item: &VerifierPreference) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPreferenceLabelLengthRule {
    pub max_length: i64,
}

impl VerifierPreferenceRule for VerifierPreferenceLabelLengthRule {
    fn check(&self, item: &VerifierPreference) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPreferenceTimezoneLengthRule {
    pub max_length: i64,
}

impl VerifierPreferenceRule for VerifierPreferenceTimezoneLengthRule {
    fn check(&self, item: &VerifierPreference) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl VerifierPreferenceRule for VerifierPreferenceCodeLengthRule {
    fn check(&self, item: &VerifierPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPreferenceRequiredCodeRule {
    pub message: String,
}

impl VerifierPreferenceRule for VerifierPreferenceRequiredCodeRule {
    fn check(&self, item: &VerifierPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierPreferenceCodePrefixRule {
    pub prefix: String,
}

impl VerifierPreferenceRule for VerifierPreferenceCodePrefixRule {
    fn check(&self, item: &VerifierPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier preference.
#[derive(Debug, Clone)]
pub struct VerifierPreferenceValidator {
    pub rules: Vec<Rc<dyn VerifierPreferenceRule>>,
}

impl VerifierPreferenceValidator {
    pub fn validate(&self, item: &VerifierPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_preference_validator() -> VerifierPreferenceValidator {
    VerifierPreferenceValidator {
        rules: vec![
            Rc::new(VerifierPreferenceFloorValueRangeRule { min: 2.4, max: 678.6 }) as Rc<dyn VerifierPreferenceRule>,
            Rc::new(VerifierPreferenceLabelLengthRule { max_length: 1000 }),
            Rc::new(VerifierPreferenceTimezoneLengthRule { max_length: 1000 }),
            Rc::new(VerifierPreferenceCodeLengthRule { max_length: 32 }),
            Rc::new(VerifierPreferenceRequiredCodeRule {
                message: "verifier preference needs this field".to_string(),
            }),
            Rc::new(VerifierPreferenceCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
