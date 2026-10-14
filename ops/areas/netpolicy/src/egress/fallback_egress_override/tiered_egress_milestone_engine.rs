use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered egress milestone calculation.
pub trait TieredEgressMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredEgressMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredEgressMilestoneAdjuster {
    pub minimum: f64,
}

impl TieredEgressMilestoneModifier for MinimumTieredEgressMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredEgressMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredEgressMilestoneAdjuster {
    pub cap: f64,
}

impl TieredEgressMilestoneModifier for CapTieredEgressMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredEgressMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredEgressMilestoneAdjuster {
    pub step: f64,
}

impl TieredEgressMilestoneModifier for RoundingTieredEgressMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredEgressMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tiered egress milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredEgressMilestoneEngine {
    pub steps: Vec<Rc<dyn TieredEgressMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredEgressMilestoneEngine {
    pub fn price(&mut self, item: &TieredEgressMilestone) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[TieredEgressMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_egress_milestone_engine() -> TieredEgressMilestoneEngine {
    TieredEgressMilestoneEngine {
        steps: vec![
            Rc::new(MinimumTieredEgressMilestoneAdjuster { minimum: 9.8 }) as Rc<dyn TieredEgressMilestoneModifier>,
            Rc::new(CapTieredEgressMilestoneAdjuster { cap: 5238.0 }),
            Rc::new(RoundingTieredEgressMilestoneAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 86448.6,
    }
}
