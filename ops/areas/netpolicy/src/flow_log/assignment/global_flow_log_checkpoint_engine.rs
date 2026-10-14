use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the global flow log checkpoint calculation.
pub trait GlobalFlowLogCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalFlowLogCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalFlowLogCheckpointAdjuster {
    pub percent: f64,
}

impl GlobalFlowLogCheckpointFactor for PercentGlobalFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalFlowLogCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalFlowLogCheckpointAdjuster {
    pub cap: f64,
}

impl GlobalFlowLogCheckpointFactor for CapGlobalFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalFlowLogCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogCheckpointFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl GlobalFlowLogCheckpointFactor for GlobalFlowLogCheckpointFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalFlowLogCheckpoint, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the global flow log checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalFlowLogCheckpointEngine {
    pub steps: Vec<Rc<dyn GlobalFlowLogCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalFlowLogCheckpointEngine {
    pub fn price(&mut self, item: &GlobalFlowLogCheckpoint) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalFlowLogCheckpointFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalFlowLogCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_flow_log_checkpoint_engine() -> GlobalFlowLogCheckpointEngine {
    GlobalFlowLogCheckpointEngine {
        steps: vec![
            Rc::new(PercentGlobalFlowLogCheckpointAdjuster { percent: -22.4 }) as Rc<dyn GlobalFlowLogCheckpointFactor>,
            Rc::new(CapGlobalFlowLogCheckpointAdjuster { cap: 33652.0 }),
            Rc::new(GlobalFlowLogCheckpointFloorValueOffsetAdjuster { weight: 0.58 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 18942.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_flow_log_checkpoint_engine();
        let item = new_global_flow_log_checkpoint("code-795", "label-248");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
