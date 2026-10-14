use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional overage entry calculation.
pub trait RegionalOverageEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalOverageEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalOverageEntryAdjuster {
    pub step: f64,
}

impl RegionalOverageEntryModifier for RoundingRegionalOverageEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalOverageEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalOverageEntryAdjuster {
    pub percent: f64,
}

impl RegionalOverageEntryModifier for PercentRegionalOverageEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalOverageEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalOverageEntryAdjuster {
    pub cap: f64,
}

impl RegionalOverageEntryModifier for CapRegionalOverageEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalOverageEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalOverageEntryAdjuster {
    pub minimum: f64,
}

impl RegionalOverageEntryModifier for MinimumRegionalOverageEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalOverageEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional overage entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalOverageEntryEngine {
    pub steps: Vec<Rc<dyn RegionalOverageEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalOverageEntryEngine {
    pub fn compute(&mut self, item: &RegionalOverageEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionalOverageEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_overage_entry_engine() -> RegionalOverageEntryEngine {
    RegionalOverageEntryEngine {
        steps: vec![
            Rc::new(RoundingRegionalOverageEntryAdjuster { step: 0.5 }) as Rc<dyn RegionalOverageEntryModifier>,
            Rc::new(PercentRegionalOverageEntryAdjuster { percent: -15.6 }),
            Rc::new(CapRegionalOverageEntryAdjuster { cap: 9493.0 }),
            Rc::new(MinimumRegionalOverageEntryAdjuster { minimum: 41.4 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 27711.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_overage_entry_engine();
        let item = new_regional_overage_entry("code-367", "title-449");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
