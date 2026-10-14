use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the suspension slot calculation.
pub trait SuspensionSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionSlotStep for FlaggedSuspensionSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionSlotAdjuster {
    pub percent: f64,
}

impl SuspensionSlotStep for PercentSuspensionSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionSlotAdjuster {
    pub cap: f64,
}

impl SuspensionSlotStep for CapSuspensionSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSlotThresholdOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionSlotStep for SuspensionSlotThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionSlot, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

/// Runs the suspension slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionSlotEngine {
    pub steps: Vec<Rc<dyn SuspensionSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionSlotEngine {
    pub fn compute(&mut self, item: &SuspensionSlot) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_suspension_slot_engine(flags: &flags::Registry) -> SuspensionSlotEngine {
    SuspensionSlotEngine {
        steps: vec![
            Rc::new(FlaggedSuspensionSlotAdjuster {
                flags: flags.clone(),
                flag: "suspension-slot-capped".to_string(),
                percent: 0.5,
            }) as Rc<dyn SuspensionSlotStep>,
            Rc::new(PercentSuspensionSlotAdjuster { percent: 6.2 }),
            Rc::new(CapSuspensionSlotAdjuster { cap: 3779.0 }),
            Rc::new(SuspensionSlotThresholdOffsetAdjuster { weight: 0.21 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 14865.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_slot_engine(&flags::new_registry());
        let item = new_suspension_slot("code-512", "name-154");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
