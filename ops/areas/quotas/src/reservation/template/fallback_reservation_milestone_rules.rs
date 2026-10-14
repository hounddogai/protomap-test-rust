use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback reservation milestone must pass. Check returns its problems, or none.
pub trait FallbackReservationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &FallbackReservationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationMilestoneQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackReservationMilestoneRule for FallbackReservationMilestoneQuantityRangeRule {
    fn check(&self, item: &FallbackReservationMilestone) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationMilestoneAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl FallbackReservationMilestoneRule for FallbackReservationMilestoneAllowedCountryRule {
    fn check(&self, item: &FallbackReservationMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is duplicated", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationMilestoneRequiredSourceRule {
    pub message: String,
}

impl FallbackReservationMilestoneRule for FallbackReservationMilestoneRequiredSourceRule {
    fn check(&self, item: &FallbackReservationMilestone) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationMilestoneMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackReservationMilestoneRule for FallbackReservationMilestoneMaximumRangeRule {
    fn check(&self, item: &FallbackReservationMilestone) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback reservation milestone.
#[derive(Debug, Clone)]
pub struct FallbackReservationMilestoneValidator {
    pub rules: Vec<Rc<dyn FallbackReservationMilestoneRule>>,
}

impl FallbackReservationMilestoneValidator {
    pub fn validate(&self, item: &FallbackReservationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackReservationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackReservationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_reservation_milestone_validator() -> FallbackReservationMilestoneValidator {
    FallbackReservationMilestoneValidator {
        rules: vec![
            Rc::new(FallbackReservationMilestoneQuantityRangeRule { min: 64, max: 3600 })
                as Rc<dyn FallbackReservationMilestoneRule>,
            Rc::new(FallbackReservationMilestoneAllowedCountryRule {
                allowed: vec!["strict".to_string(), "capped".to_string()],
            }),
            Rc::new(FallbackReservationMilestoneRequiredSourceRule {
                message: "fallback reservation milestone needs an entry".to_string(),
            }),
            Rc::new(FallbackReservationMilestoneMaximumRangeRule { min: 9.4, max: 3968.9 }),
        ],
    }
}
