use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending identity checkpoint calculation.
pub trait PendingIdentityCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingIdentityCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingIdentityCheckpointAdjuster {
    pub percent: f64,
}

impl PendingIdentityCheckpointFactor for PercentPendingIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingIdentityCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryPendingIdentityCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingIdentityCheckpointFactor for CountryPendingIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &PendingIdentityCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingIdentityCheckpointAdjuster {
    pub minimum: f64,
}

impl PendingIdentityCheckpointFactor for MinimumPendingIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingIdentityCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingIdentityCheckpointAdjuster {
    pub step: f64,
}

impl PendingIdentityCheckpointFactor for RoundingPendingIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingIdentityCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingIdentityCheckpointAdjuster {
    pub cap: f64,
}

impl PendingIdentityCheckpointFactor for CapPendingIdentityCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingIdentityCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending identity checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingIdentityCheckpointEngine {
    pub steps: Vec<Rc<dyn PendingIdentityCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingIdentityCheckpointEngine {
    pub fn compute(&mut self, item: &PendingIdentityCheckpoint) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingIdentityCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_identity_checkpoint_engine() -> PendingIdentityCheckpointEngine {
    PendingIdentityCheckpointEngine {
        steps: vec![
            Rc::new(PercentPendingIdentityCheckpointAdjuster { percent: -3.2 })
                as Rc<dyn PendingIdentityCheckpointFactor>,
            Rc::new(CountryPendingIdentityCheckpointAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.21), ("central".to_string(), 1.28)]),
            }),
            Rc::new(MinimumPendingIdentityCheckpointAdjuster { minimum: 2.0 }),
            Rc::new(RoundingPendingIdentityCheckpointAdjuster { step: 5.0 }),
            Rc::new(CapPendingIdentityCheckpointAdjuster { cap: 23074.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 74648.3,
    }
}
