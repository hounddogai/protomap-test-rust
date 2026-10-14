use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual verifier preference must pass. Check returns its problems, or none.
pub trait ManualVerifierPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ManualVerifierPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl ManualVerifierPreferenceRule for ManualVerifierPreferenceCodeLengthRule {
    fn check(&self, item: &ManualVerifierPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierPreferencePinnedGuardRule {
    pub limit: i64,
}

impl ManualVerifierPreferenceRule for ManualVerifierPreferencePinnedGuardRule {
    fn check(&self, item: &ManualVerifierPreference) -> Vec<String> {
        if item.pinned && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while pinned",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl ManualVerifierPreferenceRule for ManualVerifierPreferenceTitleLengthRule {
    fn check(&self, item: &ManualVerifierPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierPreferenceRequiredCodeRule {
    pub message: String,
}

impl ManualVerifierPreferenceRule for ManualVerifierPreferenceRequiredCodeRule {
    fn check(&self, item: &ManualVerifierPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierPreferenceRequiredDescriptionRule {
    pub message: String,
}

impl ManualVerifierPreferenceRule for ManualVerifierPreferenceRequiredDescriptionRule {
    fn check(&self, item: &ManualVerifierPreference) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual verifier preference.
#[derive(Debug, Clone)]
pub struct ManualVerifierPreferenceValidator {
    pub rules: Vec<Rc<dyn ManualVerifierPreferenceRule>>,
}

impl ManualVerifierPreferenceValidator {
    pub fn validate(&self, item: &ManualVerifierPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualVerifierPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualVerifierPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_verifier_preference_validator() -> ManualVerifierPreferenceValidator {
    ManualVerifierPreferenceValidator {
        rules: vec![
            Rc::new(ManualVerifierPreferenceCodeLengthRule { max_length: 80 }) as Rc<dyn ManualVerifierPreferenceRule>,
            Rc::new(ManualVerifierPreferencePinnedGuardRule { limit: 1000 }),
            Rc::new(ManualVerifierPreferenceTitleLengthRule { max_length: 80 }),
            Rc::new(ManualVerifierPreferenceRequiredCodeRule {
                message: "manual verifier preference needs an entry".to_string(),
            }),
            Rc::new(ManualVerifierPreferenceRequiredDescriptionRule {
                message: "manual verifier preference needs an entry".to_string(),
            }),
        ],
    }
}
