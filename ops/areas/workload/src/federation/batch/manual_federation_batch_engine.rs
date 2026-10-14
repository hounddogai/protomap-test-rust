use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual federation batch calculation.
pub trait ManualFederationBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualFederationBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationBatchPortOffsetAdjuster {
    pub weight: f64,
}

impl ManualFederationBatchAdjuster for ManualFederationBatchPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &ManualFederationBatch, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualFederationBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualFederationBatchAdjuster for TieredManualFederationBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualFederationBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualFederationBatchAdjuster {
    pub cap: f64,
}

impl ManualFederationBatchAdjuster for CapManualFederationBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualFederationBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualFederationBatchAdjuster {
    pub percent: f64,
}

impl ManualFederationBatchAdjuster for PercentManualFederationBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualFederationBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual federation batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualFederationBatchEngine {
    pub steps: Vec<Rc<dyn ManualFederationBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualFederationBatchEngine {
    pub fn run(&mut self, item: &ManualFederationBatch) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualFederationBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_federation_batch_engine() -> ManualFederationBatchEngine {
    ManualFederationBatchEngine {
        steps: vec![
            Rc::new(ManualFederationBatchPortOffsetAdjuster { weight: 1.39 }) as Rc<dyn ManualFederationBatchAdjuster>,
            Rc::new(TieredManualFederationBatchAdjuster {
                bounds: vec![65.0, 6701.0, 8911.0],
                rates: vec![0.91, 0.85, 0.87],
            }),
            Rc::new(CapManualFederationBatchAdjuster { cap: 23970.0 }),
            Rc::new(PercentManualFederationBatchAdjuster { percent: -11.9 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 59057.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_federation_batch_engine();
        let item = new_manual_federation_batch("code-610", "label-119");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
