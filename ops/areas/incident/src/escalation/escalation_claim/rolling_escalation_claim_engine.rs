use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling escalation claim calculation.
pub trait RollingEscalationClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingEscalationClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingEscalationClaimAdjuster {
    pub percent: f64,
}

impl RollingEscalationClaimModifier for PercentRollingEscalationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingEscalationClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRollingEscalationClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingEscalationClaimModifier for CategoryRollingEscalationClaimAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RollingEscalationClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingEscalationClaimAdjuster {
    pub step: f64,
}

impl RollingEscalationClaimModifier for RoundingRollingEscalationClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingEscalationClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingEscalationClaimAdjuster {
    pub minimum: f64,
}

impl RollingEscalationClaimModifier for MinimumRollingEscalationClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingEscalationClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationClaimTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl RollingEscalationClaimModifier for RollingEscalationClaimTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &RollingEscalationClaim, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingEscalationClaimAdjuster {
    pub cap: f64,
}

impl RollingEscalationClaimModifier for CapRollingEscalationClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingEscalationClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling escalation claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingEscalationClaimEngine {
    pub steps: Vec<Rc<dyn RollingEscalationClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingEscalationClaimEngine {
    pub fn evaluate(&mut self, item: &RollingEscalationClaim) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingEscalationClaimModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RollingEscalationClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_escalation_claim_engine() -> RollingEscalationClaimEngine {
    RollingEscalationClaimEngine {
        steps: vec![
            Rc::new(PercentRollingEscalationClaimAdjuster { percent: 18.0 }) as Rc<dyn RollingEscalationClaimModifier>,
            Rc::new(CategoryRollingEscalationClaimAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.12)]),
            }),
            Rc::new(RoundingRollingEscalationClaimAdjuster { step: 0.05 }),
            Rc::new(MinimumRollingEscalationClaimAdjuster { minimum: 47.3 }),
            Rc::new(RollingEscalationClaimTtlSecondsOffsetAdjuster { weight: 0.15 }),
            Rc::new(CapRollingEscalationClaimAdjuster { cap: 40065.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 68477.5,
    }
}
