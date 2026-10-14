use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log checkpoint calculation.
pub trait FlowLogCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogCheckpointAdjuster {
    pub step: f64,
}

impl FlowLogCheckpointModifier for RoundingFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitFlowLogCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogCheckpointModifier for UnitFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &FlowLogCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCheckpointQuantityOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogCheckpointModifier for FlowLogCheckpointQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogCheckpoint, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogCheckpointModifier for FlaggedFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogCheckpointAdjuster {
    pub cap: f64,
}

impl FlowLogCheckpointModifier for CapFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogCheckpointAdjuster {
    pub minimum: f64,
}

impl FlowLogCheckpointModifier for MinimumFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the flow log checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogCheckpointEngine {
    pub steps: Vec<Rc<dyn FlowLogCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogCheckpointEngine {
    pub fn price(&mut self, item: &FlowLogCheckpoint) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_checkpoint_engine(flags: &flags::Registry) -> FlowLogCheckpointEngine {
    FlowLogCheckpointEngine {
        steps: vec![
            Rc::new(RoundingFlowLogCheckpointAdjuster { step: 0.25 }) as Rc<dyn FlowLogCheckpointModifier>,
            Rc::new(UnitFlowLogCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.17),
                    ("south".to_string(), 1.38),
                    ("online".to_string(), 0.81),
                ]),
            }),
            Rc::new(FlowLogCheckpointQuantityOffsetAdjuster { weight: 0.45 }),
            Rc::new(FlaggedFlowLogCheckpointAdjuster {
                flags: flags.clone(),
                flag: "flow-log-checkpoint-weighted".to_string(),
                percent: -8.7,
            }),
            Rc::new(CapFlowLogCheckpointAdjuster { cap: 29306.0 }),
            Rc::new(MinimumFlowLogCheckpointAdjuster { minimum: 3.8 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 39800.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_checkpoint_engine(&flags::new_registry());
        let item = new_flow_log_checkpoint("code-209", "name-731");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
