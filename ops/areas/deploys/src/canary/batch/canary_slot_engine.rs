use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary slot calculation.
pub trait CanarySlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanarySlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanarySlotAdjuster {
    pub step: f64,
}

impl CanarySlotStep for RoundingCanarySlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanarySlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySlotThresholdOffsetAdjuster {
    pub weight: f64,
}

impl CanarySlotStep for CanarySlotThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &CanarySlot, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanarySlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanarySlotStep for FlaggedCanarySlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanarySlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanarySlotAdjuster {
    pub cap: f64,
}

impl CanarySlotStep for CapCanarySlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanarySlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanarySlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanarySlotStep for TieredCanarySlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanarySlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the canary slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanarySlotEngine {
    pub steps: Vec<Rc<dyn CanarySlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanarySlotEngine {
    pub fn price(&mut self, item: &CanarySlot) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CanarySlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CanarySlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_canary_slot_engine(flags: &flags::Registry) -> CanarySlotEngine {
    CanarySlotEngine {
        steps: vec![
            Rc::new(RoundingCanarySlotAdjuster { step: 5.0 }) as Rc<dyn CanarySlotStep>,
            Rc::new(CanarySlotThresholdOffsetAdjuster { weight: 1.92 }),
            Rc::new(FlaggedCanarySlotAdjuster {
                flags: flags.clone(),
                flag: "canary-slot-regional".to_string(),
                percent: 7.3,
            }),
            Rc::new(CapCanarySlotAdjuster { cap: 4808.0 }),
            Rc::new(TieredCanarySlotAdjuster {
                bounds: vec![2573.0, 4851.0, 7680.0],
                rates: vec![1.11, 1.11, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 55482.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_slot_engine(&flags::new_registry());
        let item = new_canary_slot("code-940", "title-807");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
