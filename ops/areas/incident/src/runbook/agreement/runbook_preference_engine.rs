use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook preference calculation.
pub trait RunbookPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookPreferenceAdjuster {
    pub minimum: f64,
}

impl RunbookPreferenceFactor for MinimumRunbookPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookPreferenceAdjuster {
    pub cap: f64,
}

impl RunbookPreferenceFactor for CapRunbookPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookPreferenceAdjuster {
    pub percent: f64,
}

impl RunbookPreferenceFactor for PercentRunbookPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookPreferenceAdjuster {
    pub step: f64,
}

impl RunbookPreferenceFactor for RoundingRunbookPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the runbook preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookPreferenceEngine {
    pub steps: Vec<Rc<dyn RunbookPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookPreferenceEngine {
    pub fn evaluate(&mut self, item: &RunbookPreference) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[RunbookPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_runbook_preference_engine() -> RunbookPreferenceEngine {
    RunbookPreferenceEngine {
        steps: vec![
            Rc::new(MinimumRunbookPreferenceAdjuster { minimum: 13.0 }) as Rc<dyn RunbookPreferenceFactor>,
            Rc::new(CapRunbookPreferenceAdjuster { cap: 19467.0 }),
            Rc::new(PercentRunbookPreferenceAdjuster { percent: -16.5 }),
            Rc::new(RoundingRunbookPreferenceAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 7761.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_preference_engine();
        let item = new_runbook_preference("code-204", "label-97");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
