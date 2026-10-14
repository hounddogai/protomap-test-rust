use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted redaction preference must pass. Check returns its problems, or none.
pub trait WeightedRedactionPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRedactionPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionPreferenceRequiredFingerprintRule {
    pub message: String,
}

impl WeightedRedactionPreferenceRule for WeightedRedactionPreferenceRequiredFingerprintRule {
    fn check(&self, item: &WeightedRedactionPreference) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionPreferenceRequiredTitleRule {
    pub message: String,
}

impl WeightedRedactionPreferenceRule for WeightedRedactionPreferenceRequiredTitleRule {
    fn check(&self, item: &WeightedRedactionPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl WeightedRedactionPreferenceRule for WeightedRedactionPreferenceCodeLengthRule {
    fn check(&self, item: &WeightedRedactionPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted redaction preference.
#[derive(Debug, Clone)]
pub struct WeightedRedactionPreferenceValidator {
    pub rules: Vec<Rc<dyn WeightedRedactionPreferenceRule>>,
}

impl WeightedRedactionPreferenceValidator {
    pub fn validate(&self, item: &WeightedRedactionPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRedactionPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRedactionPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_redaction_preference_validator() -> WeightedRedactionPreferenceValidator {
    WeightedRedactionPreferenceValidator {
        rules: vec![
            Rc::new(WeightedRedactionPreferenceRequiredFingerprintRule {
                message: "weighted redaction preference needs a value".to_string(),
            }) as Rc<dyn WeightedRedactionPreferenceRule>,
            Rc::new(WeightedRedactionPreferenceRequiredTitleRule {
                message: "weighted redaction preference needs a value".to_string(),
            }),
            Rc::new(WeightedRedactionPreferenceCodeLengthRule { max_length: 80 }),
        ],
    }
}
