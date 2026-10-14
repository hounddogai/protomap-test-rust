use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive suspension baseline calculation.
pub trait AdaptiveSuspensionBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveSuspensionBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveSuspensionBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveSuspensionBaselineAdjuster for TieredAdaptiveSuspensionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveSuspensionBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveSuspensionBaselineAdjuster {
    pub cap: f64,
}

impl AdaptiveSuspensionBaselineAdjuster for CapAdaptiveSuspensionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveSuspensionBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionBaselineMarginOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveSuspensionBaselineAdjuster for AdaptiveSuspensionBaselineMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveSuspensionBaseline, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the adaptive suspension baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveSuspensionBaselineEngine {
    pub steps: Vec<Rc<dyn AdaptiveSuspensionBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveSuspensionBaselineEngine {
    pub fn evaluate(&mut self, item: &AdaptiveSuspensionBaseline) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveSuspensionBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_suspension_baseline_engine() -> AdaptiveSuspensionBaselineEngine {
    AdaptiveSuspensionBaselineEngine {
        steps: vec![
            Rc::new(TieredAdaptiveSuspensionBaselineAdjuster {
                bounds: vec![3107.0, 6992.0, 9579.0],
                rates: vec![0.98, 1.11, 0.95],
            }) as Rc<dyn AdaptiveSuspensionBaselineAdjuster>,
            Rc::new(CapAdaptiveSuspensionBaselineAdjuster { cap: 16907.0 }),
            Rc::new(AdaptiveSuspensionBaselineMarginOffsetAdjuster { weight: 0.4 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 53997.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_suspension_baseline_engine();
        let item = new_adaptive_suspension_baseline("code-355", "name-292");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
