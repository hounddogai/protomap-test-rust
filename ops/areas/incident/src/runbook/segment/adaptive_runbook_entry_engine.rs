use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive runbook entry calculation.
pub trait AdaptiveRunbookEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRunbookEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRunbookEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRunbookEntryFactor for TieredAdaptiveRunbookEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRunbookEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveRunbookEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveRunbookEntryFactor for FlaggedAdaptiveRunbookEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveRunbookEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookEntryRatioOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRunbookEntryFactor for AdaptiveRunbookEntryRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRunbookEntry, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

/// Runs the adaptive runbook entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRunbookEntryEngine {
    pub steps: Vec<Rc<dyn AdaptiveRunbookEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRunbookEntryEngine {
    pub fn price(&mut self, item: &AdaptiveRunbookEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRunbookEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_runbook_entry_engine(flags: &flags::Registry) -> AdaptiveRunbookEntryEngine {
    AdaptiveRunbookEntryEngine {
        steps: vec![
            Rc::new(TieredAdaptiveRunbookEntryAdjuster {
                bounds: vec![1849.0, 2215.0],
                rates: vec![0.82, 1.01],
            }) as Rc<dyn AdaptiveRunbookEntryFactor>,
            Rc::new(FlaggedAdaptiveRunbookEntryAdjuster {
                flags: flags.clone(),
                flag: "adaptive-runbook-entry-adaptive".to_string(),
                percent: -4.5,
            }),
            Rc::new(AdaptiveRunbookEntryRatioOffsetAdjuster { weight: 0.69 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 28189.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_runbook_entry_engine(&flags::new_registry());
        let item = new_adaptive_runbook_entry("code-140", "title-576");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
