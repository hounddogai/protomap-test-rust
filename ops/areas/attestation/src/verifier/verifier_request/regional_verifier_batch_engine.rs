use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional verifier batch calculation.
pub trait RegionalVerifierBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalVerifierBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalVerifierBatchAdjuster {
    pub cap: f64,
}

impl RegionalVerifierBatchFactor for CapRegionalVerifierBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalVerifierBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalVerifierBatchAdjuster {
    pub percent: f64,
}

impl RegionalVerifierBatchFactor for PercentRegionalVerifierBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalVerifierBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalVerifierBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalVerifierBatchFactor for TieredRegionalVerifierBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalVerifierBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional verifier batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalVerifierBatchEngine {
    pub steps: Vec<Rc<dyn RegionalVerifierBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalVerifierBatchEngine {
    pub fn price(&mut self, item: &RegionalVerifierBatch) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[RegionalVerifierBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_verifier_batch_engine() -> RegionalVerifierBatchEngine {
    RegionalVerifierBatchEngine {
        steps: vec![
            Rc::new(CapRegionalVerifierBatchAdjuster { cap: 42337.0 }) as Rc<dyn RegionalVerifierBatchFactor>,
            Rc::new(PercentRegionalVerifierBatchAdjuster { percent: 13.4 }),
            Rc::new(TieredRegionalVerifierBatchAdjuster {
                bounds: vec![4455.0, 5581.0, 6372.0],
                rates: vec![0.97, 0.9, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 20115.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_verifier_batch_engine();
        let item = new_regional_verifier_batch("code-423", "label-949");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
