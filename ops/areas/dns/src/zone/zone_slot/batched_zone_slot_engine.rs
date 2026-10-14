use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched zone slot calculation.
pub trait BatchedZoneSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedZoneSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedZoneSlotAdjuster {
    pub percent: f64,
}

impl BatchedZoneSlotAdjuster for PercentBatchedZoneSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedZoneSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneSlotCapacityOffsetAdjuster {
    pub weight: f64,
}

impl BatchedZoneSlotAdjuster for BatchedZoneSlotCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedZoneSlot, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedZoneSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedZoneSlotAdjuster for FlaggedBatchedZoneSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedZoneSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedZoneSlotAdjuster {
    pub minimum: f64,
}

impl BatchedZoneSlotAdjuster for MinimumBatchedZoneSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedZoneSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched zone slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedZoneSlotEngine {
    pub steps: Vec<Rc<dyn BatchedZoneSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedZoneSlotEngine {
    pub fn compute(&mut self, item: &BatchedZoneSlot) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[BatchedZoneSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_zone_slot_engine(flags: &flags::Registry) -> BatchedZoneSlotEngine {
    BatchedZoneSlotEngine {
        steps: vec![
            Rc::new(PercentBatchedZoneSlotAdjuster { percent: 1.7 }) as Rc<dyn BatchedZoneSlotAdjuster>,
            Rc::new(BatchedZoneSlotCapacityOffsetAdjuster { weight: 0.18 }),
            Rc::new(FlaggedBatchedZoneSlotAdjuster {
                flags: flags.clone(),
                flag: "batched-zone-slot-adaptive".to_string(),
                percent: -4.4,
            }),
            Rc::new(MinimumBatchedZoneSlotAdjuster { minimum: 11.3 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 75135.0,
    }
}
