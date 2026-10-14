use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback isolation entry calculation.
pub trait FallbackIsolationEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackIsolationEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationEntryBudgetOffsetAdjuster {
    pub weight: f64,
}

impl FallbackIsolationEntryModifier for FallbackIsolationEntryBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackIsolationEntry, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackIsolationEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackIsolationEntryModifier for TieredFallbackIsolationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackIsolationEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackIsolationEntryAdjuster {
    pub step: f64,
}

impl FallbackIsolationEntryModifier for RoundingFallbackIsolationEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackIsolationEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackIsolationEntryAdjuster {
    pub minimum: f64,
}

impl FallbackIsolationEntryModifier for MinimumFallbackIsolationEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackIsolationEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback isolation entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackIsolationEntryEngine {
    pub steps: Vec<Rc<dyn FallbackIsolationEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackIsolationEntryEngine {
    pub fn compute(&mut self, item: &FallbackIsolationEntry) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackIsolationEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_isolation_entry_engine() -> FallbackIsolationEntryEngine {
    FallbackIsolationEntryEngine {
        steps: vec![
            Rc::new(FallbackIsolationEntryBudgetOffsetAdjuster { weight: 0.36 })
                as Rc<dyn FallbackIsolationEntryModifier>,
            Rc::new(TieredFallbackIsolationEntryAdjuster {
                bounds: vec![4389.0, 9525.0],
                rates: vec![1.04, 1.2],
            }),
            Rc::new(RoundingFallbackIsolationEntryAdjuster { step: 0.1 }),
            Rc::new(MinimumFallbackIsolationEntryAdjuster { minimum: 17.3 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 6130.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_isolation_entry_engine();
        let item = new_fallback_isolation_entry("code-153", "name-639");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
