use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive query adjustment calculation.
pub trait AdaptiveQueryAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveQueryAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAdaptiveQueryAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveQueryAdjustmentModifier for TitleAdaptiveQueryAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AdaptiveQueryAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveQueryAdjustmentAdjuster {
    pub step: f64,
}

impl AdaptiveQueryAdjustmentModifier for RoundingAdaptiveQueryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveQueryAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveQueryAdjustmentAdjuster {
    pub cap: f64,
}

impl AdaptiveQueryAdjustmentModifier for CapAdaptiveQueryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveQueryAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveQueryAdjustmentAdjuster {
    pub minimum: f64,
}

impl AdaptiveQueryAdjustmentModifier for MinimumAdaptiveQueryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveQueryAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveQueryAdjustmentAdjuster {
    pub percent: f64,
}

impl AdaptiveQueryAdjustmentModifier for PercentAdaptiveQueryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveQueryAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustmentMinimumOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveQueryAdjustmentModifier for AdaptiveQueryAdjustmentMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveQueryAdjustment, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the adaptive query adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveQueryAdjustmentEngine {
    pub steps: Vec<Rc<dyn AdaptiveQueryAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveQueryAdjustmentEngine {
    pub fn price(&mut self, item: &AdaptiveQueryAdjustment) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveQueryAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_query_adjustment_engine() -> AdaptiveQueryAdjustmentEngine {
    AdaptiveQueryAdjustmentEngine {
        steps: vec![
            Rc::new(TitleAdaptiveQueryAdjustmentAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.84)]),
            }) as Rc<dyn AdaptiveQueryAdjustmentModifier>,
            Rc::new(RoundingAdaptiveQueryAdjustmentAdjuster { step: 5.0 }),
            Rc::new(CapAdaptiveQueryAdjustmentAdjuster { cap: 45309.0 }),
            Rc::new(MinimumAdaptiveQueryAdjustmentAdjuster { minimum: 22.7 }),
            Rc::new(PercentAdaptiveQueryAdjustmentAdjuster { percent: 23.8 }),
            Rc::new(AdaptiveQueryAdjustmentMinimumOffsetAdjuster { weight: 1.91 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 47194.8,
    }
}
