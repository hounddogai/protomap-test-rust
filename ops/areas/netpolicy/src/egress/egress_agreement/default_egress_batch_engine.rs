use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default egress batch calculation.
pub trait DefaultEgressBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultEgressBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultEgressBatchAdjuster {
    pub cap: f64,
}

impl DefaultEgressBatchModifier for CapDefaultEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultEgressBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultEgressBatchAdjuster {
    pub percent: f64,
}

impl DefaultEgressBatchModifier for PercentDefaultEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultEgressBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultEgressBatchAdjuster {
    pub step: f64,
}

impl DefaultEgressBatchModifier for RoundingDefaultEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultEgressBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultEgressBatchAdjuster {
    pub minimum: f64,
}

impl DefaultEgressBatchModifier for MinimumDefaultEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultEgressBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionDefaultEgressBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultEgressBatchModifier for RegionDefaultEgressBatchAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &DefaultEgressBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultEgressBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultEgressBatchModifier for TieredDefaultEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultEgressBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default egress batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultEgressBatchEngine {
    pub steps: Vec<Rc<dyn DefaultEgressBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultEgressBatchEngine {
    pub fn run(&mut self, item: &DefaultEgressBatch) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultEgressBatchModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[DefaultEgressBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_egress_batch_engine() -> DefaultEgressBatchEngine {
    DefaultEgressBatchEngine {
        steps: vec![
            Rc::new(CapDefaultEgressBatchAdjuster { cap: 45368.0 }) as Rc<dyn DefaultEgressBatchModifier>,
            Rc::new(PercentDefaultEgressBatchAdjuster { percent: -12.4 }),
            Rc::new(RoundingDefaultEgressBatchAdjuster { step: 0.05 }),
            Rc::new(MinimumDefaultEgressBatchAdjuster { minimum: 42.5 }),
            Rc::new(RegionDefaultEgressBatchAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.26)]),
            }),
            Rc::new(TieredDefaultEgressBatchAdjuster {
                bounds: vec![39.0, 7409.0, 8322.0, 9867.0],
                rates: vec![1.03, 0.89, 0.99, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 50367.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_egress_batch_engine();
        let item = new_default_egress_batch("code-35", "label-626");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
