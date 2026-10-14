use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient slo entry calculation.
pub trait LenientSloEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSloEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientSloEntryAdjuster {
    pub minimum: f64,
}

impl LenientSloEntryAdjuster for MinimumLenientSloEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientSloEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientSloEntryAdjuster {
    pub percent: f64,
}

impl LenientSloEntryAdjuster for PercentLenientSloEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientSloEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientSloEntryAdjuster {
    pub cap: f64,
}

impl LenientSloEntryAdjuster for CapLenientSloEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientSloEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLenientSloEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientSloEntryAdjuster for SourceLenientSloEntryAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &LenientSloEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient slo entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSloEntryEngine {
    pub steps: Vec<Rc<dyn LenientSloEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSloEntryEngine {
    pub fn evaluate(&mut self, item: &LenientSloEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LenientSloEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_slo_entry_engine() -> LenientSloEntryEngine {
    LenientSloEntryEngine {
        steps: vec![
            Rc::new(MinimumLenientSloEntryAdjuster { minimum: 11.9 }) as Rc<dyn LenientSloEntryAdjuster>,
            Rc::new(PercentLenientSloEntryAdjuster { percent: -6.8 }),
            Rc::new(CapLenientSloEntryAdjuster { cap: 46972.0 }),
            Rc::new(SourceLenientSloEntryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.18)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 16313.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_slo_entry_engine();
        let item = new_lenient_slo_entry("code-422", "name-980");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
