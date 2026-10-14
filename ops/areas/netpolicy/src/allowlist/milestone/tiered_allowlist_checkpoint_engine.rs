use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered allowlist checkpoint calculation.
pub trait TieredAllowlistCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredAllowlistCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredAllowlistCheckpointAdjuster {
    pub percent: f64,
}

impl TieredAllowlistCheckpointStep for PercentTieredAllowlistCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredAllowlistCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredAllowlistCheckpointAdjuster {
    pub step: f64,
}

impl TieredAllowlistCheckpointStep for RoundingTieredAllowlistCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredAllowlistCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTieredAllowlistCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredAllowlistCheckpointStep for LocaleTieredAllowlistCheckpointAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TieredAllowlistCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredAllowlistCheckpointAdjuster {
    pub minimum: f64,
}

impl TieredAllowlistCheckpointStep for MinimumTieredAllowlistCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredAllowlistCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered allowlist checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredAllowlistCheckpointEngine {
    pub steps: Vec<Rc<dyn TieredAllowlistCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredAllowlistCheckpointEngine {
    pub fn evaluate(&mut self, item: &TieredAllowlistCheckpoint) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredAllowlistCheckpointStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredAllowlistCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_allowlist_checkpoint_engine() -> TieredAllowlistCheckpointEngine {
    TieredAllowlistCheckpointEngine {
        steps: vec![
            Rc::new(PercentTieredAllowlistCheckpointAdjuster { percent: 17.7 })
                as Rc<dyn TieredAllowlistCheckpointStep>,
            Rc::new(RoundingTieredAllowlistCheckpointAdjuster { step: 5.0 }),
            Rc::new(LocaleTieredAllowlistCheckpointAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.37)]),
            }),
            Rc::new(MinimumTieredAllowlistCheckpointAdjuster { minimum: 31.2 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 7623.6,
    }
}
