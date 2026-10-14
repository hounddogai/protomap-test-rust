use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional query variance must pass. Check returns its problems, or none.
pub trait RegionalQueryVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RegionalQueryVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryVarianceCodePrefixRule {
    pub prefix: String,
}

impl RegionalQueryVarianceRule for RegionalQueryVarianceCodePrefixRule {
    fn check(&self, item: &RegionalQueryVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryVarianceRequiredCodeRule {
    pub message: String,
}

impl RegionalQueryVarianceRule for RegionalQueryVarianceRequiredCodeRule {
    fn check(&self, item: &RegionalQueryVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryVarianceRequiredLabelRule {
    pub message: String,
}

impl RegionalQueryVarianceRule for RegionalQueryVarianceRequiredLabelRule {
    fn check(&self, item: &RegionalQueryVariance) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryVarianceRequiredHostRule {
    pub message: String,
}

impl RegionalQueryVarianceRule for RegionalQueryVarianceRequiredHostRule {
    fn check(&self, item: &RegionalQueryVariance) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional query variance.
#[derive(Debug, Clone)]
pub struct RegionalQueryVarianceValidator {
    pub rules: Vec<Rc<dyn RegionalQueryVarianceRule>>,
}

impl RegionalQueryVarianceValidator {
    pub fn validate(&self, item: &RegionalQueryVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalQueryVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalQueryVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_query_variance_validator() -> RegionalQueryVarianceValidator {
    RegionalQueryVarianceValidator {
        rules: vec![
            Rc::new(RegionalQueryVarianceCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn RegionalQueryVarianceRule>,
            Rc::new(RegionalQueryVarianceRequiredCodeRule {
                message: "regional query variance needs this field".to_string(),
            }),
            Rc::new(RegionalQueryVarianceRequiredLabelRule {
                message: "regional query variance needs a value".to_string(),
            }),
            Rc::new(RegionalQueryVarianceRequiredHostRule {
                message: "regional query variance needs this field".to_string(),
            }),
        ],
    }
}
