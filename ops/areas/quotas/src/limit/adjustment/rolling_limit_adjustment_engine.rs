use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling limit adjustment calculation.
pub trait RollingLimitAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingLimitAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitAdjustmentRatioOffsetAdjuster {
    pub weight: f64,
}

impl RollingLimitAdjustmentStep for RollingLimitAdjustmentRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &RollingLimitAdjustment, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingLimitAdjustmentAdjuster {
    pub minimum: f64,
}

impl RollingLimitAdjustmentStep for MinimumRollingLimitAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingLimitAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingLimitAdjustmentAdjuster {
    pub cap: f64,
}

impl RollingLimitAdjustmentStep for CapRollingLimitAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingLimitAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling limit adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingLimitAdjustmentEngine {
    pub steps: Vec<Rc<dyn RollingLimitAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingLimitAdjustmentEngine {
    pub fn price(&mut self, item: &RollingLimitAdjustment) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[RollingLimitAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_limit_adjustment_engine() -> RollingLimitAdjustmentEngine {
    RollingLimitAdjustmentEngine {
        steps: vec![
            Rc::new(RollingLimitAdjustmentRatioOffsetAdjuster { weight: 1.69 }) as Rc<dyn RollingLimitAdjustmentStep>,
            Rc::new(MinimumRollingLimitAdjustmentAdjuster { minimum: 34.1 }),
            Rc::new(CapRollingLimitAdjustmentAdjuster { cap: 17233.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 59979.2,
    }
}
