use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted query manifest must pass. Check returns its problems, or none.
pub trait WeightedQueryManifestRule: std::fmt::Debug {
    fn check(&self, item: &WeightedQueryManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryManifestSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedQueryManifestRule for WeightedQueryManifestSequenceRangeRule {
    fn check(&self, item: &WeightedQueryManifest) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryManifestRequiredLabelRule {
    pub message: String,
}

impl WeightedQueryManifestRule for WeightedQueryManifestRequiredLabelRule {
    fn check(&self, item: &WeightedQueryManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryManifestLabelLengthRule {
    pub max_length: i64,
}

impl WeightedQueryManifestRule for WeightedQueryManifestLabelLengthRule {
    fn check(&self, item: &WeightedQueryManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryManifestCodeLengthRule {
    pub max_length: i64,
}

impl WeightedQueryManifestRule for WeightedQueryManifestCodeLengthRule {
    fn check(&self, item: &WeightedQueryManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryManifestCodePrefixRule {
    pub prefix: String,
}

impl WeightedQueryManifestRule for WeightedQueryManifestCodePrefixRule {
    fn check(&self, item: &WeightedQueryManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryManifestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedQueryManifestRule for WeightedQueryManifestCreatedAtRangeRule {
    fn check(&self, item: &WeightedQueryManifest) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted query manifest.
#[derive(Debug, Clone)]
pub struct WeightedQueryManifestValidator {
    pub rules: Vec<Rc<dyn WeightedQueryManifestRule>>,
}

impl WeightedQueryManifestValidator {
    pub fn validate(&self, item: &WeightedQueryManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedQueryManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedQueryManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_query_manifest_validator() -> WeightedQueryManifestValidator {
    WeightedQueryManifestValidator {
        rules: vec![
            Rc::new(WeightedQueryManifestSequenceRangeRule { min: 32, max: 80 }) as Rc<dyn WeightedQueryManifestRule>,
            Rc::new(WeightedQueryManifestRequiredLabelRule {
                message: "weighted query manifest needs a value".to_string(),
            }),
            Rc::new(WeightedQueryManifestLabelLengthRule { max_length: 80 }),
            Rc::new(WeightedQueryManifestCodeLengthRule { max_length: 255 }),
            Rc::new(WeightedQueryManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedQueryManifestCreatedAtRangeRule { min: 255, max: 1000 }),
        ],
    }
}
