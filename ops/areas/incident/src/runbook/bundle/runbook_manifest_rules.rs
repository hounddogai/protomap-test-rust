use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook manifest must pass. Check returns its problems, or none.
pub trait RunbookManifestRule: std::fmt::Debug {
    fn check(&self, item: &RunbookManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookManifestAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookManifestRule for RunbookManifestAmountRangeRule {
    fn check(&self, item: &RunbookManifest) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookManifestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RunbookManifestRule for RunbookManifestAllowedTitleRule {
    fn check(&self, item: &RunbookManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookManifestRequiredLocaleRule {
    pub message: String,
}

impl RunbookManifestRule for RunbookManifestRequiredLocaleRule {
    fn check(&self, item: &RunbookManifest) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookManifestRequiredTitleRule {
    pub message: String,
}

impl RunbookManifestRule for RunbookManifestRequiredTitleRule {
    fn check(&self, item: &RunbookManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookManifestRequiredCodeRule {
    pub message: String,
}

impl RunbookManifestRule for RunbookManifestRequiredCodeRule {
    fn check(&self, item: &RunbookManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookManifestCodePrefixRule {
    pub prefix: String,
}

impl RunbookManifestRule for RunbookManifestCodePrefixRule {
    fn check(&self, item: &RunbookManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook manifest.
#[derive(Debug, Clone)]
pub struct RunbookManifestValidator {
    pub rules: Vec<Rc<dyn RunbookManifestRule>>,
}

impl RunbookManifestValidator {
    pub fn validate(&self, item: &RunbookManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_manifest_validator() -> RunbookManifestValidator {
    RunbookManifestValidator {
        rules: vec![
            Rc::new(RunbookManifestAmountRangeRule { min: 6.3, max: 1989.5 }) as Rc<dyn RunbookManifestRule>,
            Rc::new(RunbookManifestAllowedTitleRule {
                allowed: vec!["global".to_string(), "default".to_string()],
            }),
            Rc::new(RunbookManifestRequiredLocaleRule {
                message: "runbook manifest needs this field".to_string(),
            }),
            Rc::new(RunbookManifestRequiredTitleRule {
                message: "runbook manifest needs this field".to_string(),
            }),
            Rc::new(RunbookManifestRequiredCodeRule {
                message: "runbook manifest needs this field".to_string(),
            }),
            Rc::new(RunbookManifestCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
