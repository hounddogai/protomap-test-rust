use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal measurement manifest must pass. Check returns its problems, or none.
pub trait SeasonalMeasurementManifestRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalMeasurementManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMeasurementManifestAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalMeasurementManifestRule for SeasonalMeasurementManifestAmountRangeRule {
    fn check(&self, item: &SeasonalMeasurementManifest) -> Vec<String> {
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
pub struct SeasonalMeasurementManifestCodePrefixRule {
    pub prefix: String,
}

impl SeasonalMeasurementManifestRule for SeasonalMeasurementManifestCodePrefixRule {
    fn check(&self, item: &SeasonalMeasurementManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMeasurementManifestRequiredLabelRule {
    pub message: String,
}

impl SeasonalMeasurementManifestRule for SeasonalMeasurementManifestRequiredLabelRule {
    fn check(&self, item: &SeasonalMeasurementManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMeasurementManifestLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalMeasurementManifestRule for SeasonalMeasurementManifestLabelLengthRule {
    fn check(&self, item: &SeasonalMeasurementManifest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMeasurementManifestVisibleGuardRule {
    pub limit: f64,
}

impl SeasonalMeasurementManifestRule for SeasonalMeasurementManifestVisibleGuardRule {
    fn check(&self, item: &SeasonalMeasurementManifest) -> Vec<String> {
        if item.visible && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while visible", item.amount, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal measurement manifest.
#[derive(Debug, Clone)]
pub struct SeasonalMeasurementManifestValidator {
    pub rules: Vec<Rc<dyn SeasonalMeasurementManifestRule>>,
}

impl SeasonalMeasurementManifestValidator {
    pub fn validate(&self, item: &SeasonalMeasurementManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalMeasurementManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalMeasurementManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_measurement_manifest_validator() -> SeasonalMeasurementManifestValidator {
    SeasonalMeasurementManifestValidator {
        rules: vec![
            Rc::new(SeasonalMeasurementManifestAmountRangeRule { min: 2.1, max: 3789.4 })
                as Rc<dyn SeasonalMeasurementManifestRule>,
            Rc::new(SeasonalMeasurementManifestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SeasonalMeasurementManifestRequiredLabelRule {
                message: "seasonal measurement manifest needs an entry".to_string(),
            }),
            Rc::new(SeasonalMeasurementManifestLabelLengthRule { max_length: 255 }),
            Rc::new(SeasonalMeasurementManifestVisibleGuardRule { limit: 4.1 }),
        ],
    }
}
