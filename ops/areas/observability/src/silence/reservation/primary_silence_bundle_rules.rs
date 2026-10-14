use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary silence bundle must pass. Check returns its problems, or none.
pub trait PrimarySilenceBundleRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySilenceBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceBundleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimarySilenceBundleRule for PrimarySilenceBundleCeilingRangeRule {
    fn check(&self, item: &PrimarySilenceBundle) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceBundleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PrimarySilenceBundleRule for PrimarySilenceBundleAllowedNameRule {
    fn check(&self, item: &PrimarySilenceBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceBundleRequiredNameRule {
    pub message: String,
}

impl PrimarySilenceBundleRule for PrimarySilenceBundleRequiredNameRule {
    fn check(&self, item: &PrimarySilenceBundle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceBundleCodePrefixRule {
    pub prefix: String,
}

impl PrimarySilenceBundleRule for PrimarySilenceBundleCodePrefixRule {
    fn check(&self, item: &PrimarySilenceBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceBundleNameLengthRule {
    pub max_length: i64,
}

impl PrimarySilenceBundleRule for PrimarySilenceBundleNameLengthRule {
    fn check(&self, item: &PrimarySilenceBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceBundleVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimarySilenceBundleRule for PrimarySilenceBundleVersionRangeRule {
    fn check(&self, item: &PrimarySilenceBundle) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a primary silence bundle.
#[derive(Debug, Clone)]
pub struct PrimarySilenceBundleValidator {
    pub rules: Vec<Rc<dyn PrimarySilenceBundleRule>>,
}

impl PrimarySilenceBundleValidator {
    pub fn validate(&self, item: &PrimarySilenceBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySilenceBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySilenceBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_silence_bundle_validator() -> PrimarySilenceBundleValidator {
    PrimarySilenceBundleValidator {
        rules: vec![
            Rc::new(PrimarySilenceBundleCeilingRangeRule { min: 5.6, max: 4045.0 }) as Rc<dyn PrimarySilenceBundleRule>,
            Rc::new(PrimarySilenceBundleAllowedNameRule {
                allowed: vec!["lenient".to_string(), "regional".to_string()],
            }),
            Rc::new(PrimarySilenceBundleRequiredNameRule {
                message: "primary silence bundle needs an entry".to_string(),
            }),
            Rc::new(PrimarySilenceBundleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PrimarySilenceBundleNameLengthRule { max_length: 16 }),
            Rc::new(PrimarySilenceBundleVersionRangeRule { min: 64, max: 120 }),
        ],
    }
}
