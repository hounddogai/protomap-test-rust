use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement rule must pass. Check returns its problems, or none.
pub trait MeasurementRuleRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRuleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl MeasurementRuleRule for MeasurementRuleAllowedLabelRule {
    fn check(&self, item: &MeasurementRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRuleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementRuleRule for MeasurementRuleRateRangeRule {
    fn check(&self, item: &MeasurementRule) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRuleRequiredCodeRule {
    pub message: String,
}

impl MeasurementRuleRule for MeasurementRuleRequiredCodeRule {
    fn check(&self, item: &MeasurementRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRuleCodePrefixRule {
    pub prefix: String,
}

impl MeasurementRuleRule for MeasurementRuleCodePrefixRule {
    fn check(&self, item: &MeasurementRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRuleRequiredLabelRule {
    pub message: String,
}

impl MeasurementRuleRule for MeasurementRuleRequiredLabelRule {
    fn check(&self, item: &MeasurementRule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRuleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementRuleRule for MeasurementRuleDiscountRangeRule {
    fn check(&self, item: &MeasurementRule) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a measurement rule.
#[derive(Debug, Clone)]
pub struct MeasurementRuleValidator {
    pub rules: Vec<Rc<dyn MeasurementRuleRule>>,
}

impl MeasurementRuleValidator {
    pub fn validate(&self, item: &MeasurementRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_rule_validator() -> MeasurementRuleValidator {
    MeasurementRuleValidator {
        rules: vec![
            Rc::new(MeasurementRuleAllowedLabelRule { allowed: Vec::new() }) as Rc<dyn MeasurementRuleRule>,
            Rc::new(MeasurementRuleRateRangeRule { min: 5.9, max: 862.5 }),
            Rc::new(MeasurementRuleRequiredCodeRule {
                message: "measurement rule needs this field".to_string(),
            }),
            Rc::new(MeasurementRuleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(MeasurementRuleRequiredLabelRule {
                message: "measurement rule needs a value".to_string(),
            }),
            Rc::new(MeasurementRuleDiscountRangeRule { min: 0.3, max: 3102.5 }),
        ],
    }
}
