use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default peering adjustment must pass. Check returns its problems, or none.
pub trait DefaultPeeringAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPeeringAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringAdjustmentRequiredLabelRule {
    pub message: String,
}

impl DefaultPeeringAdjustmentRule for DefaultPeeringAdjustmentRequiredLabelRule {
    fn check(&self, item: &DefaultPeeringAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringAdjustmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultPeeringAdjustmentRule for DefaultPeeringAdjustmentFloorValueRangeRule {
    fn check(&self, item: &DefaultPeeringAdjustment) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl DefaultPeeringAdjustmentRule for DefaultPeeringAdjustmentCodePrefixRule {
    fn check(&self, item: &DefaultPeeringAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default peering adjustment.
#[derive(Debug, Clone)]
pub struct DefaultPeeringAdjustmentValidator {
    pub rules: Vec<Rc<dyn DefaultPeeringAdjustmentRule>>,
}

impl DefaultPeeringAdjustmentValidator {
    pub fn validate(&self, item: &DefaultPeeringAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultPeeringAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPeeringAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_peering_adjustment_validator() -> DefaultPeeringAdjustmentValidator {
    DefaultPeeringAdjustmentValidator {
        rules: vec![
            Rc::new(DefaultPeeringAdjustmentRequiredLabelRule {
                message: "default peering adjustment needs a value".to_string(),
            }) as Rc<dyn DefaultPeeringAdjustmentRule>,
            Rc::new(DefaultPeeringAdjustmentFloorValueRangeRule { min: 3.7, max: 2322.5 }),
            Rc::new(DefaultPeeringAdjustmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
