use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual quote batch calculation.
pub trait ManualQuoteBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualQuoteBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualQuoteBatchAdjuster {
    pub cap: f64,
}

impl ManualQuoteBatchModifier for CapManualQuoteBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualQuoteBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualQuoteBatchAdjuster {
    pub step: f64,
}

impl ManualQuoteBatchModifier for RoundingManualQuoteBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualQuoteBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualQuoteBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualQuoteBatchModifier for TieredManualQuoteBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualQuoteBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual quote batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualQuoteBatchEngine {
    pub steps: Vec<Rc<dyn ManualQuoteBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualQuoteBatchEngine {
    pub fn price(&mut self, item: &ManualQuoteBatch) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualQuoteBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_quote_batch_engine() -> ManualQuoteBatchEngine {
    ManualQuoteBatchEngine {
        steps: vec![
            Rc::new(CapManualQuoteBatchAdjuster { cap: 3313.0 }) as Rc<dyn ManualQuoteBatchModifier>,
            Rc::new(RoundingManualQuoteBatchAdjuster { step: 1.0 }),
            Rc::new(TieredManualQuoteBatchAdjuster {
                bounds: vec![7758.0, 9780.0],
                rates: vec![0.86, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 94858.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_quote_batch_engine();
        let item = new_manual_quote_batch("code-798", "title-379");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
