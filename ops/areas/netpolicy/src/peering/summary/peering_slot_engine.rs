use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the peering slot calculation.
pub trait PeeringSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringSlotAdjuster {
    pub percent: f64,
}

impl PeeringSlotAdjuster for PercentPeeringSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSlotEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl PeeringSlotAdjuster for PeeringSlotEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringSlot, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringSlotAdjuster for FlaggedPeeringSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringSlotAdjuster {
    pub minimum: f64,
}

impl PeeringSlotAdjuster for MinimumPeeringSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the peering slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringSlotEngine {
    pub steps: Vec<Rc<dyn PeeringSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringSlotEngine {
    pub fn price(&mut self, item: &PeeringSlot) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringSlotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_peering_slot_engine(flags: &flags::Registry) -> PeeringSlotEngine {
    PeeringSlotEngine {
        steps: vec![
            Rc::new(PercentPeeringSlotAdjuster { percent: -18.3 }) as Rc<dyn PeeringSlotAdjuster>,
            Rc::new(PeeringSlotEndsAtOffsetAdjuster { weight: 0.87 }),
            Rc::new(FlaggedPeeringSlotAdjuster {
                flags: flags.clone(),
                flag: "peering-slot-regional".to_string(),
                percent: 9.3,
            }),
            Rc::new(MinimumPeeringSlotAdjuster { minimum: 29.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 69584.9,
    }
}
