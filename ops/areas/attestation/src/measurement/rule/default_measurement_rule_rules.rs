use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default measurement rule must pass. Check returns its problems, or none.
pub trait DefaultMeasurementRuleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultMeasurementRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementRuleTitleLengthRule {
    pub max_length: i64,
}

impl DefaultMeasurementRuleRule for DefaultMeasurementRuleTitleLengthRule {
    fn check(&self, item: &DefaultMeasurementRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementRuleCodePrefixRule {
    pub prefix: String,
}

impl DefaultMeasurementRuleRule for DefaultMeasurementRuleCodePrefixRule {
    fn check(&self, item: &DefaultMeasurementRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementRuleNotesLengthRule {
    pub max_length: i64,
}

impl DefaultMeasurementRuleRule for DefaultMeasurementRuleNotesLengthRule {
    fn check(&self, item: &DefaultMeasurementRule) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementRuleRegionLengthRule {
    pub max_length: i64,
}

impl DefaultMeasurementRuleRule for DefaultMeasurementRuleRegionLengthRule {
    fn check(&self, item: &DefaultMeasurementRule) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default measurement rule.
#[derive(Debug, Clone)]
pub struct DefaultMeasurementRuleValidator {
    pub rules: Vec<Rc<dyn DefaultMeasurementRuleRule>>,
}

impl DefaultMeasurementRuleValidator {
    pub fn validate(&self, item: &DefaultMeasurementRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultMeasurementRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultMeasurementRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_measurement_rule_validator() -> DefaultMeasurementRuleValidator {
    DefaultMeasurementRuleValidator {
        rules: vec![
            Rc::new(DefaultMeasurementRuleTitleLengthRule { max_length: 80 }) as Rc<dyn DefaultMeasurementRuleRule>,
            Rc::new(DefaultMeasurementRuleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultMeasurementRuleNotesLengthRule { max_length: 80 }),
            Rc::new(DefaultMeasurementRuleRegionLengthRule { max_length: 120 }),
        ],
    }
}
