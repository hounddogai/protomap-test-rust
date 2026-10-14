use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered egress cycle calculation.
pub trait TieredEgressCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredEgressCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredEgressCycleAdjuster {
    pub step: f64,
}

impl TieredEgressCycleAdjuster for RoundingTieredEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredEgressCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredEgressCycleAdjuster {
    pub percent: f64,
}

impl TieredEgressCycleAdjuster for PercentTieredEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredEgressCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredEgressCycleAdjuster {
    pub cap: f64,
}

impl TieredEgressCycleAdjuster for CapTieredEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredEgressCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tiered egress cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredEgressCycleEngine {
    pub steps: Vec<Rc<dyn TieredEgressCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredEgressCycleEngine {
    pub fn evaluate(&mut self, item: &TieredEgressCycle) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TieredEgressCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_egress_cycle_engine() -> TieredEgressCycleEngine {
    TieredEgressCycleEngine {
        steps: vec![
            Rc::new(RoundingTieredEgressCycleAdjuster { step: 0.05 }) as Rc<dyn TieredEgressCycleAdjuster>,
            Rc::new(PercentTieredEgressCycleAdjuster { percent: 0.3 }),
            Rc::new(CapTieredEgressCycleAdjuster { cap: 46853.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 56385.0,
    }
}
