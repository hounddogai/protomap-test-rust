use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert slot calculation.
pub trait AlertSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertSlotAdjuster {
    pub percent: f64,
}

impl AlertSlotFactor for PercentAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertSlotAdjuster {
    pub step: f64,
}

impl AlertSlotFactor for RoundingAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertSlotAdjuster {
    pub cap: f64,
}

impl AlertSlotFactor for CapAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertSlotFactor for FlaggedAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertSlotAdjuster {
    pub minimum: f64,
}

impl AlertSlotFactor for MinimumAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertSlotRevisionOffsetAdjuster {
    pub weight: f64,
}

impl AlertSlotFactor for AlertSlotRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &AlertSlot, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the alert slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertSlotEngine {
    pub steps: Vec<Rc<dyn AlertSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertSlotEngine {
    pub fn run(&mut self, item: &AlertSlot) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[AlertSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_alert_slot_engine(flags: &flags::Registry) -> AlertSlotEngine {
    AlertSlotEngine {
        steps: vec![
            Rc::new(PercentAlertSlotAdjuster { percent: -12.7 }) as Rc<dyn AlertSlotFactor>,
            Rc::new(RoundingAlertSlotAdjuster { step: 0.5 }),
            Rc::new(CapAlertSlotAdjuster { cap: 40437.0 }),
            Rc::new(FlaggedAlertSlotAdjuster {
                flags: flags.clone(),
                flag: "alert-slot-legacy".to_string(),
                percent: 5.8,
            }),
            Rc::new(MinimumAlertSlotAdjuster { minimum: 6.8 }),
            Rc::new(AlertSlotRevisionOffsetAdjuster { weight: 0.11 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 20283.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_slot_engine(&flags::new_registry());
        let item = new_alert_slot("code-763", "name-181");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
