use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted zone checkpoint calculation.
pub trait WeightedZoneCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedZoneCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedZoneCheckpointAdjuster {
    pub step: f64,
}

impl WeightedZoneCheckpointFactor for RoundingWeightedZoneCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedZoneCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedZoneCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedZoneCheckpointFactor for TieredWeightedZoneCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedZoneCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedZoneCheckpointAdjuster {
    pub minimum: f64,
}

impl WeightedZoneCheckpointFactor for MinimumWeightedZoneCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedZoneCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneCheckpointRevisionOffsetAdjuster {
    pub weight: f64,
}

impl WeightedZoneCheckpointFactor for WeightedZoneCheckpointRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedZoneCheckpoint, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedZoneCheckpointAdjuster {
    pub cap: f64,
}

impl WeightedZoneCheckpointFactor for CapWeightedZoneCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedZoneCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedZoneCheckpointAdjuster {
    pub percent: f64,
}

impl WeightedZoneCheckpointFactor for PercentWeightedZoneCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedZoneCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted zone checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedZoneCheckpointEngine {
    pub steps: Vec<Rc<dyn WeightedZoneCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedZoneCheckpointEngine {
    pub fn price(&mut self, item: &WeightedZoneCheckpoint) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedZoneCheckpointFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedZoneCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_zone_checkpoint_engine() -> WeightedZoneCheckpointEngine {
    WeightedZoneCheckpointEngine {
        steps: vec![
            Rc::new(RoundingWeightedZoneCheckpointAdjuster { step: 0.1 }) as Rc<dyn WeightedZoneCheckpointFactor>,
            Rc::new(TieredWeightedZoneCheckpointAdjuster {
                bounds: vec![6157.0, 7600.0],
                rates: vec![1.1, 0.97],
            }),
            Rc::new(MinimumWeightedZoneCheckpointAdjuster { minimum: 10.4 }),
            Rc::new(WeightedZoneCheckpointRevisionOffsetAdjuster { weight: 0.6 }),
            Rc::new(CapWeightedZoneCheckpointAdjuster { cap: 4700.0 }),
            Rc::new(PercentWeightedZoneCheckpointAdjuster { percent: 3.7 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 23913.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_zone_checkpoint_engine();
        let item = new_weighted_zone_checkpoint("code-102", "label-444");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
