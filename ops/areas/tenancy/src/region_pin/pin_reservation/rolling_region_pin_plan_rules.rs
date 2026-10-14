use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling region pin plan must pass. Check returns its problems, or none.
pub trait RollingRegionPinPlanRule: std::fmt::Debug {
    fn check(&self, item: &RollingRegionPinPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlanAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl RollingRegionPinPlanRule for RollingRegionPinPlanAllowedCurrencyRule {
    fn check(&self, item: &RollingRegionPinPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is stale", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlanRequiredCurrencyRule {
    pub message: String,
}

impl RollingRegionPinPlanRule for RollingRegionPinPlanRequiredCurrencyRule {
    fn check(&self, item: &RollingRegionPinPlan) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlanCurrencyLengthRule {
    pub max_length: i64,
}

impl RollingRegionPinPlanRule for RollingRegionPinPlanCurrencyLengthRule {
    fn check(&self, item: &RollingRegionPinPlan) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlanCodeLengthRule {
    pub max_length: i64,
}

impl RollingRegionPinPlanRule for RollingRegionPinPlanCodeLengthRule {
    fn check(&self, item: &RollingRegionPinPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlanThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingRegionPinPlanRule for RollingRegionPinPlanThresholdRangeRule {
    fn check(&self, item: &RollingRegionPinPlan) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlanRequiredSourceRule {
    pub message: String,
}

impl RollingRegionPinPlanRule for RollingRegionPinPlanRequiredSourceRule {
    fn check(&self, item: &RollingRegionPinPlan) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling region pin plan.
#[derive(Debug, Clone)]
pub struct RollingRegionPinPlanValidator {
    pub rules: Vec<Rc<dyn RollingRegionPinPlanRule>>,
}

impl RollingRegionPinPlanValidator {
    pub fn validate(&self, item: &RollingRegionPinPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingRegionPinPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRegionPinPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_region_pin_plan_validator() -> RollingRegionPinPlanValidator {
    RollingRegionPinPlanValidator {
        rules: vec![
            Rc::new(RollingRegionPinPlanAllowedCurrencyRule { allowed: Vec::new() })
                as Rc<dyn RollingRegionPinPlanRule>,
            Rc::new(RollingRegionPinPlanRequiredCurrencyRule {
                message: "rolling region pin plan needs this field".to_string(),
            }),
            Rc::new(RollingRegionPinPlanCurrencyLengthRule { max_length: 80 }),
            Rc::new(RollingRegionPinPlanCodeLengthRule { max_length: 1000 }),
            Rc::new(RollingRegionPinPlanThresholdRangeRule { min: 3.6, max: 3940.6 }),
            Rc::new(RollingRegionPinPlanRequiredSourceRule {
                message: "rolling region pin plan needs a value".to_string(),
            }),
        ],
    }
}
