use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback region pin plan must pass. Check returns its problems, or none.
pub trait FallbackRegionPinPlanRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRegionPinPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinPlanDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackRegionPinPlanRule for FallbackRegionPinPlanDiscountRangeRule {
    fn check(&self, item: &FallbackRegionPinPlan) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinPlanCipherLengthRule {
    pub max_length: i64,
}

impl FallbackRegionPinPlanRule for FallbackRegionPinPlanCipherLengthRule {
    fn check(&self, item: &FallbackRegionPinPlan) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinPlanCodeLengthRule {
    pub max_length: i64,
}

impl FallbackRegionPinPlanRule for FallbackRegionPinPlanCodeLengthRule {
    fn check(&self, item: &FallbackRegionPinPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinPlanAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl FallbackRegionPinPlanRule for FallbackRegionPinPlanAllowedLabelRule {
    fn check(&self, item: &FallbackRegionPinPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinPlanRequiredCodeRule {
    pub message: String,
}

impl FallbackRegionPinPlanRule for FallbackRegionPinPlanRequiredCodeRule {
    fn check(&self, item: &FallbackRegionPinPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback region pin plan.
#[derive(Debug, Clone)]
pub struct FallbackRegionPinPlanValidator {
    pub rules: Vec<Rc<dyn FallbackRegionPinPlanRule>>,
}

impl FallbackRegionPinPlanValidator {
    pub fn validate(&self, item: &FallbackRegionPinPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackRegionPinPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRegionPinPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_region_pin_plan_validator() -> FallbackRegionPinPlanValidator {
    FallbackRegionPinPlanValidator {
        rules: vec![
            Rc::new(FallbackRegionPinPlanDiscountRangeRule { min: 9.5, max: 3287.3 })
                as Rc<dyn FallbackRegionPinPlanRule>,
            Rc::new(FallbackRegionPinPlanCipherLengthRule { max_length: 3600 }),
            Rc::new(FallbackRegionPinPlanCodeLengthRule { max_length: 120 }),
            Rc::new(FallbackRegionPinPlanAllowedLabelRule {
                allowed: vec!["fallback".to_string(), "fallback".to_string(), "seasonal".to_string()],
            }),
            Rc::new(FallbackRegionPinPlanRequiredCodeRule {
                message: "fallback region pin plan needs a value".to_string(),
            }),
        ],
    }
}
