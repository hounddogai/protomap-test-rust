use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered probe checkpoint calculation.
pub trait TieredProbeCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredProbeCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeCheckpointLimitOffsetAdjuster {
    pub weight: f64,
}

impl TieredProbeCheckpointAdjuster for TieredProbeCheckpointLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &TieredProbeCheckpoint, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTieredProbeCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredProbeCheckpointAdjuster for NameTieredProbeCheckpointAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TieredProbeCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredProbeCheckpointAdjuster {
    pub cap: f64,
}

impl TieredProbeCheckpointAdjuster for CapTieredProbeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredProbeCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredProbeCheckpointAdjuster {
    pub minimum: f64,
}

impl TieredProbeCheckpointAdjuster for MinimumTieredProbeCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredProbeCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered probe checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredProbeCheckpointEngine {
    pub steps: Vec<Rc<dyn TieredProbeCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredProbeCheckpointEngine {
    pub fn run(&mut self, item: &TieredProbeCheckpoint) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[TieredProbeCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_probe_checkpoint_engine() -> TieredProbeCheckpointEngine {
    TieredProbeCheckpointEngine {
        steps: vec![
            Rc::new(TieredProbeCheckpointLimitOffsetAdjuster { weight: 1.81 }) as Rc<dyn TieredProbeCheckpointAdjuster>,
            Rc::new(NameTieredProbeCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.97),
                    ("retail".to_string(), 1.37),
                    ("south".to_string(), 1.13),
                ]),
            }),
            Rc::new(CapTieredProbeCheckpointAdjuster { cap: 45728.0 }),
            Rc::new(MinimumTieredProbeCheckpointAdjuster { minimum: 1.7 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 58228.6,
    }
}
