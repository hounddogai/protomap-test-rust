use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal quote manifest must pass. Check returns its problems, or none.
pub trait SeasonalQuoteManifestRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalQuoteManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteManifestRequiredUnitRule {
    pub message: String,
}

impl SeasonalQuoteManifestRule for SeasonalQuoteManifestRequiredUnitRule {
    fn check(&self, item: &SeasonalQuoteManifest) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteManifestLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalQuoteManifestRule for SeasonalQuoteManifestLabelLengthRule {
    fn check(&self, item: &SeasonalQuoteManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteManifestRequiredCodeRule {
    pub message: String,
}

impl SeasonalQuoteManifestRule for SeasonalQuoteManifestRequiredCodeRule {
    fn check(&self, item: &SeasonalQuoteManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteManifestCodePrefixRule {
    pub prefix: String,
}

impl SeasonalQuoteManifestRule for SeasonalQuoteManifestCodePrefixRule {
    fn check(&self, item: &SeasonalQuoteManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteManifestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalQuoteManifestRule for SeasonalQuoteManifestCeilingRangeRule {
    fn check(&self, item: &SeasonalQuoteManifest) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalQuoteManifestCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalQuoteManifestRule for SeasonalQuoteManifestCodeLengthRule {
    fn check(&self, item: &SeasonalQuoteManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal quote manifest.
#[derive(Debug, Clone)]
pub struct SeasonalQuoteManifestValidator {
    pub rules: Vec<Rc<dyn SeasonalQuoteManifestRule>>,
}

impl SeasonalQuoteManifestValidator {
    pub fn validate(&self, item: &SeasonalQuoteManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalQuoteManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalQuoteManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_quote_manifest_validator() -> SeasonalQuoteManifestValidator {
    SeasonalQuoteManifestValidator {
        rules: vec![
            Rc::new(SeasonalQuoteManifestRequiredUnitRule {
                message: "seasonal quote manifest needs this field".to_string(),
            }) as Rc<dyn SeasonalQuoteManifestRule>,
            Rc::new(SeasonalQuoteManifestLabelLengthRule { max_length: 120 }),
            Rc::new(SeasonalQuoteManifestRequiredCodeRule {
                message: "seasonal quote manifest needs a value".to_string(),
            }),
            Rc::new(SeasonalQuoteManifestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SeasonalQuoteManifestCeilingRangeRule { min: 4.5, max: 517.1 }),
            Rc::new(SeasonalQuoteManifestCodeLengthRule { max_length: 1000 }),
        ],
    }
}
