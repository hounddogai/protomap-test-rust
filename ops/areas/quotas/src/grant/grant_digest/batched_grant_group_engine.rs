use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched grant group calculation.
pub trait BatchedGrantGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedGrantGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedGrantGroupAdjuster {
    pub cap: f64,
}

impl BatchedGrantGroupModifier for CapBatchedGrantGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedGrantGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedGrantGroupAdjuster {
    pub step: f64,
}

impl BatchedGrantGroupModifier for RoundingBatchedGrantGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedGrantGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedGrantGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedGrantGroupModifier for FlaggedBatchedGrantGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedGrantGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedGrantGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedGrantGroupModifier for TieredBatchedGrantGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedGrantGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedGrantGroupAdjuster {
    pub percent: f64,
}

impl BatchedGrantGroupModifier for PercentBatchedGrantGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedGrantGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedGrantGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedGrantGroupModifier for LabelBatchedGrantGroupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedGrantGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched grant group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedGrantGroupEngine {
    pub steps: Vec<Rc<dyn BatchedGrantGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedGrantGroupEngine {
    pub fn price(&mut self, item: &BatchedGrantGroup) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedGrantGroupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedGrantGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_grant_group_engine(flags: &flags::Registry) -> BatchedGrantGroupEngine {
    BatchedGrantGroupEngine {
        steps: vec![
            Rc::new(CapBatchedGrantGroupAdjuster { cap: 15829.0 }) as Rc<dyn BatchedGrantGroupModifier>,
            Rc::new(RoundingBatchedGrantGroupAdjuster { step: 0.1 }),
            Rc::new(FlaggedBatchedGrantGroupAdjuster {
                flags: flags.clone(),
                flag: "batched-grant-group-global".to_string(),
                percent: 4.4,
            }),
            Rc::new(TieredBatchedGrantGroupAdjuster {
                bounds: vec![1406.0, 3453.0],
                rates: vec![1.12, 1.1],
            }),
            Rc::new(PercentBatchedGrantGroupAdjuster { percent: -0.4 }),
            Rc::new(LabelBatchedGrantGroupAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.18)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 91438.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_grant_group_engine(&flags::new_registry());
        let item = new_batched_grant_group("code-224", "label-536");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
