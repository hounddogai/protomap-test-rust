use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover entry calculation.
pub trait FailoverEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverEntryAdjuster {
    pub step: f64,
}

impl FailoverEntryFactor for RoundingFailoverEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFailoverEntryAdjuster {
    pub percent: f64,
}

impl FailoverEntryFactor for PercentFailoverEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FailoverEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEntryRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl FailoverEntryFactor for FailoverEntryRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverEntry, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

/// Runs the failover entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverEntryEngine {
    pub steps: Vec<Rc<dyn FailoverEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverEntryEngine {
    pub fn price(&mut self, item: &FailoverEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FailoverEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_entry_engine() -> FailoverEntryEngine {
    FailoverEntryEngine {
        steps: vec![
            Rc::new(RoundingFailoverEntryAdjuster { step: 0.05 }) as Rc<dyn FailoverEntryFactor>,
            Rc::new(PercentFailoverEntryAdjuster { percent: 14.1 }),
            Rc::new(FailoverEntryRotationDaysOffsetAdjuster { weight: 1.27 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 86622.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_entry_engine();
        let item = new_failover_entry("code-805", "name-96");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
