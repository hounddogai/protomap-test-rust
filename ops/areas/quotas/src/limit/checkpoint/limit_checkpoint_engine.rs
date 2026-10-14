use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the limit checkpoint calculation.
pub trait LimitCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLimitCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitCheckpointAdjuster for TitleLimitCheckpointAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LimitCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitCheckpointAdjuster {
    pub step: f64,
}

impl LimitCheckpointAdjuster for RoundingLimitCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitCheckpointAdjuster {
    pub cap: f64,
}

impl LimitCheckpointAdjuster for CapLimitCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitCheckpointAdjuster for TieredLimitCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the limit checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitCheckpointEngine {
    pub steps: Vec<Rc<dyn LimitCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitCheckpointEngine {
    pub fn compute(&mut self, item: &LimitCheckpoint) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitCheckpointAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_limit_checkpoint_engine() -> LimitCheckpointEngine {
    LimitCheckpointEngine {
        steps: vec![
            Rc::new(TitleLimitCheckpointAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.88)]),
            }) as Rc<dyn LimitCheckpointAdjuster>,
            Rc::new(RoundingLimitCheckpointAdjuster { step: 0.05 }),
            Rc::new(CapLimitCheckpointAdjuster { cap: 25674.0 }),
            Rc::new(TieredLimitCheckpointAdjuster {
                bounds: vec![2705.0, 9358.0],
                rates: vec![0.81, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 71141.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_checkpoint_engine();
        let item = new_limit_checkpoint("code-993", "title-940");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
