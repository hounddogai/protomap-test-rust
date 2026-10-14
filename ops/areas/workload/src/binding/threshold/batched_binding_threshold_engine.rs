use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched binding threshold calculation.
pub trait BatchedBindingThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedBindingThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedBindingThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedBindingThresholdFactor for FlaggedBatchedBindingThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedBindingThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedBindingThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedBindingThresholdFactor for LabelBatchedBindingThresholdAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedBindingThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingThresholdUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl BatchedBindingThresholdFactor for BatchedBindingThresholdUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedBindingThreshold, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedBindingThresholdAdjuster {
    pub minimum: f64,
}

impl BatchedBindingThresholdFactor for MinimumBatchedBindingThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedBindingThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedBindingThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedBindingThresholdFactor for TieredBatchedBindingThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedBindingThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedBindingThresholdAdjuster {
    pub percent: f64,
}

impl BatchedBindingThresholdFactor for PercentBatchedBindingThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedBindingThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched binding threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedBindingThresholdEngine {
    pub steps: Vec<Rc<dyn BatchedBindingThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedBindingThresholdEngine {
    pub fn price(&mut self, item: &BatchedBindingThreshold) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedBindingThresholdFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedBindingThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_binding_threshold_engine(flags: &flags::Registry) -> BatchedBindingThresholdEngine {
    BatchedBindingThresholdEngine {
        steps: vec![
            Rc::new(FlaggedBatchedBindingThresholdAdjuster {
                flags: flags.clone(),
                flag: "batched-binding-threshold-scheduled".to_string(),
                percent: -9.1,
            }) as Rc<dyn BatchedBindingThresholdFactor>,
            Rc::new(LabelBatchedBindingThresholdAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.24), ("west".to_string(), 1.18)]),
            }),
            Rc::new(BatchedBindingThresholdUpdatedAtOffsetAdjuster { weight: 0.77 }),
            Rc::new(MinimumBatchedBindingThresholdAdjuster { minimum: 13.0 }),
            Rc::new(TieredBatchedBindingThresholdAdjuster {
                bounds: vec![3602.0, 3635.0, 4259.0, 9867.0],
                rates: vec![0.97, 1.01, 1.2, 1.01],
            }),
            Rc::new(PercentBatchedBindingThresholdAdjuster { percent: 22.9 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 7590.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_binding_threshold_engine(&flags::new_registry());
        let item = new_batched_binding_threshold("code-107", "label-835");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
