use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation checkpoint calculation.
pub trait IsolationCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationCheckpointRatioOffsetAdjuster {
    pub weight: f64,
}

impl IsolationCheckpointAdjuster for IsolationCheckpointRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationCheckpoint, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationCheckpointAdjuster {
    pub step: f64,
}

impl IsolationCheckpointAdjuster for RoundingIsolationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationCheckpointAdjuster {
    pub percent: f64,
}

impl IsolationCheckpointAdjuster for PercentIsolationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationCheckpointAdjuster for TieredIsolationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationCheckpointAdjuster for FlaggedIsolationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the isolation checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationCheckpointEngine {
    pub steps: Vec<Rc<dyn IsolationCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationCheckpointEngine {
    pub fn run(&mut self, item: &IsolationCheckpoint) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IsolationCheckpointAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IsolationCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_isolation_checkpoint_engine(flags: &flags::Registry) -> IsolationCheckpointEngine {
    IsolationCheckpointEngine {
        steps: vec![
            Rc::new(IsolationCheckpointRatioOffsetAdjuster { weight: 0.47 }) as Rc<dyn IsolationCheckpointAdjuster>,
            Rc::new(RoundingIsolationCheckpointAdjuster { step: 0.05 }),
            Rc::new(PercentIsolationCheckpointAdjuster { percent: -21.7 }),
            Rc::new(TieredIsolationCheckpointAdjuster {
                bounds: vec![854.0, 4854.0],
                rates: vec![1.11, 1.14],
            }),
            Rc::new(FlaggedIsolationCheckpointAdjuster {
                flags: flags.clone(),
                flag: "isolation-checkpoint-default".to_string(),
                percent: 1.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 51704.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_checkpoint_engine(&flags::new_registry());
        let item = new_isolation_checkpoint("code-538", "name-280");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
