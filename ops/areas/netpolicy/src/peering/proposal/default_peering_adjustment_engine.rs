use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default peering adjustment calculation.
pub trait DefaultPeeringAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultPeeringAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultPeeringAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultPeeringAdjustmentStep for TieredDefaultPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultPeeringAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultPeeringAdjustmentAdjuster {
    pub minimum: f64,
}

impl DefaultPeeringAdjustmentStep for MinimumDefaultPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultPeeringAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringAdjustmentQuantityOffsetAdjuster {
    pub weight: f64,
}

impl DefaultPeeringAdjustmentStep for DefaultPeeringAdjustmentQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultPeeringAdjustment, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the default peering adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultPeeringAdjustmentEngine {
    pub steps: Vec<Rc<dyn DefaultPeeringAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultPeeringAdjustmentEngine {
    pub fn price(&mut self, item: &DefaultPeeringAdjustment) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[DefaultPeeringAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_peering_adjustment_engine() -> DefaultPeeringAdjustmentEngine {
    DefaultPeeringAdjustmentEngine {
        steps: vec![
            Rc::new(TieredDefaultPeeringAdjustmentAdjuster {
                bounds: vec![6040.0, 7674.0],
                rates: vec![0.81, 0.99],
            }) as Rc<dyn DefaultPeeringAdjustmentStep>,
            Rc::new(MinimumDefaultPeeringAdjustmentAdjuster { minimum: 0.3 }),
            Rc::new(DefaultPeeringAdjustmentQuantityOffsetAdjuster { weight: 0.69 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 44529.1,
    }
}
