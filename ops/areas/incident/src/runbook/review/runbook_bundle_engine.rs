use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook bundle calculation.
pub trait RunbookBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRunbookBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookBundleStep for CountryRunbookBundleAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RunbookBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookBundleAdjuster {
    pub minimum: f64,
}

impl RunbookBundleStep for MinimumRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RunbookBundleStep for TieredRunbookBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RunbookBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the runbook bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookBundleEngine {
    pub steps: Vec<Rc<dyn RunbookBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookBundleEngine {
    pub fn evaluate(&mut self, item: &RunbookBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RunbookBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_runbook_bundle_engine() -> RunbookBundleEngine {
    RunbookBundleEngine {
        steps: vec![
            Rc::new(CountryRunbookBundleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.79)]),
            }) as Rc<dyn RunbookBundleStep>,
            Rc::new(MinimumRunbookBundleAdjuster { minimum: 48.1 }),
            Rc::new(TieredRunbookBundleAdjuster {
                bounds: vec![5416.0, 9614.0, 9667.0],
                rates: vec![0.81, 1.1, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 37840.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_bundle_engine();
        let item = new_runbook_bundle("code-596", "label-626");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
