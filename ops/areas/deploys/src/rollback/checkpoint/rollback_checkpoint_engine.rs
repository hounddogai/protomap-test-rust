use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollback checkpoint calculation.
pub trait RollbackCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackCheckpointStep for FlaggedRollbackCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackCheckpointAdjuster {
    pub step: f64,
}

impl RollbackCheckpointStep for RoundingRollbackCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackCheckpointAdjuster {
    pub minimum: f64,
}

impl RollbackCheckpointStep for MinimumRollbackCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rollback checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackCheckpointEngine {
    pub steps: Vec<Rc<dyn RollbackCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackCheckpointEngine {
    pub fn price(&mut self, item: &RollbackCheckpoint) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackCheckpointStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollback_checkpoint_engine(flags: &flags::Registry) -> RollbackCheckpointEngine {
    RollbackCheckpointEngine {
        steps: vec![
            Rc::new(FlaggedRollbackCheckpointAdjuster {
                flags: flags.clone(),
                flag: "rollback-checkpoint-tiered".to_string(),
                percent: 5.9,
            }) as Rc<dyn RollbackCheckpointStep>,
            Rc::new(RoundingRollbackCheckpointAdjuster { step: 0.05 }),
            Rc::new(MinimumRollbackCheckpointAdjuster { minimum: 2.5 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 52547.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_checkpoint_engine(&flags::new_registry());
        let item = new_rollback_checkpoint("code-796", "name-528");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
