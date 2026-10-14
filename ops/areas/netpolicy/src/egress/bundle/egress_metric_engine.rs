use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress metric calculation.
pub trait EgressMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressMetricModifier for TieredEgressMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressMetricAdjuster {
    pub step: f64,
}

impl EgressMetricModifier for RoundingEgressMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressMetricAdjuster {
    pub cap: f64,
}

impl EgressMetricModifier for CapEgressMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressMetricAdjuster {
    pub percent: f64,
}

impl EgressMetricModifier for PercentEgressMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressMetricAdjuster {
    pub minimum: f64,
}

impl EgressMetricModifier for MinimumEgressMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the egress metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressMetricEngine {
    pub steps: Vec<Rc<dyn EgressMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressMetricEngine {
    pub fn run(&mut self, item: &EgressMetric) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressMetricModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_egress_metric_engine() -> EgressMetricEngine {
    EgressMetricEngine {
        steps: vec![
            Rc::new(TieredEgressMetricAdjuster {
                bounds: vec![1993.0, 5569.0],
                rates: vec![1.1, 0.8],
            }) as Rc<dyn EgressMetricModifier>,
            Rc::new(RoundingEgressMetricAdjuster { step: 0.5 }),
            Rc::new(CapEgressMetricAdjuster { cap: 20339.0 }),
            Rc::new(PercentEgressMetricAdjuster { percent: 3.4 }),
            Rc::new(MinimumEgressMetricAdjuster { minimum: 47.1 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 58002.1,
    }
}
