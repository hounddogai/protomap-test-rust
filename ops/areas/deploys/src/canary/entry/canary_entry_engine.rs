use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary entry calculation.
pub trait CanaryEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryEntryFactor for TieredCanaryEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryEntryAdjuster {
    pub cap: f64,
}

impl CanaryEntryFactor for CapCanaryEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryEntryAdjuster {
    pub step: f64,
}

impl CanaryEntryFactor for RoundingCanaryEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the canary entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryEntryEngine {
    pub steps: Vec<Rc<dyn CanaryEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryEntryEngine {
    pub fn price(&mut self, item: &CanaryEntry) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[CanaryEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_canary_entry_engine() -> CanaryEntryEngine {
    CanaryEntryEngine {
        steps: vec![
            Rc::new(TieredCanaryEntryAdjuster {
                bounds: vec![52.0, 9798.0],
                rates: vec![0.87, 0.83],
            }) as Rc<dyn CanaryEntryFactor>,
            Rc::new(CapCanaryEntryAdjuster { cap: 13797.0 }),
            Rc::new(RoundingCanaryEntryAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 16996.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_entry_engine();
        let item = new_canary_entry("code-797", "label-800");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
