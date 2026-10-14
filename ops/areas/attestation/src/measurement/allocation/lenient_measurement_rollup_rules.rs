use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient measurement rollup must pass. Check returns its problems, or none.
pub trait LenientMeasurementRollupRule: std::fmt::Debug {
    fn check(&self, item: &LenientMeasurementRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementRollupIssuerLengthRule {
    pub max_length: i64,
}

impl LenientMeasurementRollupRule for LenientMeasurementRollupIssuerLengthRule {
    fn check(&self, item: &LenientMeasurementRollup) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementRollupRequiredLabelRule {
    pub message: String,
}

impl LenientMeasurementRollupRule for LenientMeasurementRollupRequiredLabelRule {
    fn check(&self, item: &LenientMeasurementRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementRollupCodeLengthRule {
    pub max_length: i64,
}

impl LenientMeasurementRollupRule for LenientMeasurementRollupCodeLengthRule {
    fn check(&self, item: &LenientMeasurementRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementRollupCodePrefixRule {
    pub prefix: String,
}

impl LenientMeasurementRollupRule for LenientMeasurementRollupCodePrefixRule {
    fn check(&self, item: &LenientMeasurementRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient measurement rollup.
#[derive(Debug, Clone)]
pub struct LenientMeasurementRollupValidator {
    pub rules: Vec<Rc<dyn LenientMeasurementRollupRule>>,
}

impl LenientMeasurementRollupValidator {
    pub fn validate(&self, item: &LenientMeasurementRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientMeasurementRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientMeasurementRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_measurement_rollup_validator() -> LenientMeasurementRollupValidator {
    LenientMeasurementRollupValidator {
        rules: vec![
            Rc::new(LenientMeasurementRollupIssuerLengthRule { max_length: 32 })
                as Rc<dyn LenientMeasurementRollupRule>,
            Rc::new(LenientMeasurementRollupRequiredLabelRule {
                message: "lenient measurement rollup needs a value".to_string(),
            }),
            Rc::new(LenientMeasurementRollupCodeLengthRule { max_length: 16 }),
            Rc::new(LenientMeasurementRollupCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
