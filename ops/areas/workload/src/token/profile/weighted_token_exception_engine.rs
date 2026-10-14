use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted token exception calculation.
pub trait WeightedTokenExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTokenException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedTokenExceptionAdjuster {
    pub minimum: f64,
}

impl WeightedTokenExceptionStep for MinimumWeightedTokenExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedTokenException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedTokenExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedTokenExceptionStep for TieredWeightedTokenExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedTokenException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedTokenExceptionAdjuster {
    pub cap: f64,
}

impl WeightedTokenExceptionStep for CapWeightedTokenExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedTokenException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted token exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTokenExceptionEngine {
    pub steps: Vec<Rc<dyn WeightedTokenExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTokenExceptionEngine {
    pub fn price(&mut self, item: &WeightedTokenException) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTokenException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_token_exception_engine() -> WeightedTokenExceptionEngine {
    WeightedTokenExceptionEngine {
        steps: vec![
            Rc::new(MinimumWeightedTokenExceptionAdjuster { minimum: 28.5 }) as Rc<dyn WeightedTokenExceptionStep>,
            Rc::new(TieredWeightedTokenExceptionAdjuster {
                bounds: vec![867.0, 2065.0, 4917.0],
                rates: vec![0.95, 1.02, 1.14],
            }),
            Rc::new(CapWeightedTokenExceptionAdjuster { cap: 40558.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 38092.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_token_exception_engine();
        let item = new_weighted_token_exception("code-280", "label-542");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
