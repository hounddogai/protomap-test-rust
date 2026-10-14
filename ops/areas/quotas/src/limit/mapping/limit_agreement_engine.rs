use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the limit agreement calculation.
pub trait LimitAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitAgreementAdjuster {
    pub step: f64,
}

impl LimitAgreementModifier for RoundingLimitAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitAgreementAdjuster {
    pub percent: f64,
}

impl LimitAgreementModifier for PercentLimitAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAgreementCostOffsetAdjuster {
    pub weight: f64,
}

impl LimitAgreementModifier for LimitAgreementCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &LimitAgreement, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitAgreementModifier for TieredLimitAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the limit agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitAgreementEngine {
    pub steps: Vec<Rc<dyn LimitAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitAgreementEngine {
    pub fn compute(&mut self, item: &LimitAgreement) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitAgreementModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_limit_agreement_engine() -> LimitAgreementEngine {
    LimitAgreementEngine {
        steps: vec![
            Rc::new(RoundingLimitAgreementAdjuster { step: 0.5 }) as Rc<dyn LimitAgreementModifier>,
            Rc::new(PercentLimitAgreementAdjuster { percent: 6.3 }),
            Rc::new(LimitAgreementCostOffsetAdjuster { weight: 1.48 }),
            Rc::new(TieredLimitAgreementAdjuster {
                bounds: vec![186.0, 4706.0, 7679.0, 9563.0],
                rates: vec![1.09, 1.12, 1.13, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 48818.0,
    }
}
