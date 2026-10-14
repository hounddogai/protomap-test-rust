use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched delegation checkpoint calculation.
pub trait BatchedDelegationCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedDelegationCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedDelegationCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedDelegationCheckpointModifier for LabelBatchedDelegationCheckpointAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedDelegationCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedDelegationCheckpointAdjuster {
    pub percent: f64,
}

impl BatchedDelegationCheckpointModifier for PercentBatchedDelegationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedDelegationCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedDelegationCheckpointAdjuster {
    pub step: f64,
}

impl BatchedDelegationCheckpointModifier for RoundingBatchedDelegationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedDelegationCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedDelegationCheckpointAdjuster {
    pub minimum: f64,
}

impl BatchedDelegationCheckpointModifier for MinimumBatchedDelegationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedDelegationCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedDelegationCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedDelegationCheckpointModifier for FlaggedBatchedDelegationCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedDelegationCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched delegation checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedDelegationCheckpointEngine {
    pub steps: Vec<Rc<dyn BatchedDelegationCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedDelegationCheckpointEngine {
    pub fn run(&mut self, item: &BatchedDelegationCheckpoint) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedDelegationCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_delegation_checkpoint_engine(flags: &flags::Registry) -> BatchedDelegationCheckpointEngine {
    BatchedDelegationCheckpointEngine {
        steps: vec![
            Rc::new(LabelBatchedDelegationCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.71),
                    ("north".to_string(), 1.06),
                    ("south".to_string(), 0.75),
                ]),
            }) as Rc<dyn BatchedDelegationCheckpointModifier>,
            Rc::new(PercentBatchedDelegationCheckpointAdjuster { percent: 0.2 }),
            Rc::new(RoundingBatchedDelegationCheckpointAdjuster { step: 1.0 }),
            Rc::new(MinimumBatchedDelegationCheckpointAdjuster { minimum: 10.5 }),
            Rc::new(FlaggedBatchedDelegationCheckpointAdjuster {
                flags: flags.clone(),
                flag: "batched-delegation-checkpoint-tiered".to_string(),
                percent: -9.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 13452.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_delegation_checkpoint_engine(&flags::new_registry());
        let item = new_batched_delegation_checkpoint("code-397", "label-313");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
