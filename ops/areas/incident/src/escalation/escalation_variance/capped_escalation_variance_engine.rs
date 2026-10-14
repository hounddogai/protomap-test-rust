use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped escalation variance calculation.
pub trait CappedEscalationVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedEscalationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedEscalationVarianceAdjuster {
    pub percent: f64,
}

impl CappedEscalationVarianceStep for PercentCappedEscalationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedEscalationVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedEscalationVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedEscalationVarianceStep for TieredCappedEscalationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedEscalationVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedEscalationVarianceAdjuster {
    pub minimum: f64,
}

impl CappedEscalationVarianceStep for MinimumCappedEscalationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedEscalationVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped escalation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedEscalationVarianceEngine {
    pub steps: Vec<Rc<dyn CappedEscalationVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedEscalationVarianceEngine {
    pub fn run(&mut self, item: &CappedEscalationVariance) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[CappedEscalationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_escalation_variance_engine() -> CappedEscalationVarianceEngine {
    CappedEscalationVarianceEngine {
        steps: vec![
            Rc::new(PercentCappedEscalationVarianceAdjuster { percent: 15.5 }) as Rc<dyn CappedEscalationVarianceStep>,
            Rc::new(TieredCappedEscalationVarianceAdjuster {
                bounds: vec![1446.0, 1592.0],
                rates: vec![1.2, 0.9],
            }),
            Rc::new(MinimumCappedEscalationVarianceAdjuster { minimum: 1.9 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 42979.5,
    }
}
