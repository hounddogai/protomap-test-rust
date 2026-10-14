use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook override calculation.
pub trait RunbookOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookOverrideAdjuster {
    pub step: f64,
}

impl RunbookOverrideModifier for RoundingRunbookOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookOverrideAdjuster {
    pub minimum: f64,
}

impl RunbookOverrideModifier for MinimumRunbookOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookOverrideAdjuster {
    pub percent: f64,
}

impl RunbookOverrideModifier for PercentRunbookOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RunbookOverrideModifier for TieredRunbookOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RunbookOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRunbookOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookOverrideModifier for ScopeRunbookOverrideAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RunbookOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the runbook override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookOverrideEngine {
    pub steps: Vec<Rc<dyn RunbookOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookOverrideEngine {
    pub fn evaluate(&mut self, item: &RunbookOverride) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RunbookOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_runbook_override_engine() -> RunbookOverrideEngine {
    RunbookOverrideEngine {
        steps: vec![
            Rc::new(RoundingRunbookOverrideAdjuster { step: 0.25 }) as Rc<dyn RunbookOverrideModifier>,
            Rc::new(MinimumRunbookOverrideAdjuster { minimum: 34.7 }),
            Rc::new(PercentRunbookOverrideAdjuster { percent: 21.5 }),
            Rc::new(TieredRunbookOverrideAdjuster {
                bounds: vec![1624.0, 7752.0, 8840.0],
                rates: vec![0.95, 1.14, 0.93],
            }),
            Rc::new(ScopeRunbookOverrideAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.8),
                    ("north".to_string(), 1.39),
                    ("retail".to_string(), 0.81),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 67478.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_override_engine();
        let item = new_runbook_override("code-2", "name-323");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
