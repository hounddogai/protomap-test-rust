use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the overage estimate calculation.
pub trait OverageEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageEstimateAdjuster {
    pub step: f64,
}

impl OverageEstimateModifier for RoundingOverageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageEstimateAdjuster {
    pub minimum: f64,
}

impl OverageEstimateModifier for MinimumOverageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEstimateBurstOffsetAdjuster {
    pub weight: f64,
}

impl OverageEstimateModifier for OverageEstimateBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &OverageEstimate, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryOverageEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageEstimateModifier for CountryOverageEstimateAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &OverageEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOverageEstimateAdjuster {
    pub percent: f64,
}

impl OverageEstimateModifier for PercentOverageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OverageEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the overage estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageEstimateEngine {
    pub steps: Vec<Rc<dyn OverageEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageEstimateEngine {
    pub fn price(&mut self, item: &OverageEstimate) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[OverageEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_overage_estimate_engine() -> OverageEstimateEngine {
    OverageEstimateEngine {
        steps: vec![
            Rc::new(RoundingOverageEstimateAdjuster { step: 0.05 }) as Rc<dyn OverageEstimateModifier>,
            Rc::new(MinimumOverageEstimateAdjuster { minimum: 15.3 }),
            Rc::new(OverageEstimateBurstOffsetAdjuster { weight: 0.36 }),
            Rc::new(CountryOverageEstimateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.78), ("south".to_string(), 0.72)]),
            }),
            Rc::new(PercentOverageEstimateAdjuster { percent: -11.8 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 82561.9,
    }
}
