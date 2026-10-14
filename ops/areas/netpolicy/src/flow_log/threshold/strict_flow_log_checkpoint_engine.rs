use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict flow log checkpoint calculation.
pub trait StrictFlowLogCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictFlowLogCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictFlowLogCheckpointAdjuster {
    pub step: f64,
}

impl StrictFlowLogCheckpointModifier for RoundingStrictFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictFlowLogCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictFlowLogCheckpointAdjuster {
    pub minimum: f64,
}

impl StrictFlowLogCheckpointModifier for MinimumStrictFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictFlowLogCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryStrictFlowLogCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictFlowLogCheckpointModifier for CategoryStrictFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &StrictFlowLogCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFlowLogCheckpointBurstOffsetAdjuster {
    pub weight: f64,
}

impl StrictFlowLogCheckpointModifier for StrictFlowLogCheckpointBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &StrictFlowLogCheckpoint, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictFlowLogCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictFlowLogCheckpointModifier for FlaggedStrictFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictFlowLogCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictFlowLogCheckpointAdjuster {
    pub cap: f64,
}

impl StrictFlowLogCheckpointModifier for CapStrictFlowLogCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictFlowLogCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict flow log checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictFlowLogCheckpointEngine {
    pub steps: Vec<Rc<dyn StrictFlowLogCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictFlowLogCheckpointEngine {
    pub fn price(&mut self, item: &StrictFlowLogCheckpoint) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictFlowLogCheckpointModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictFlowLogCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_flow_log_checkpoint_engine(flags: &flags::Registry) -> StrictFlowLogCheckpointEngine {
    StrictFlowLogCheckpointEngine {
        steps: vec![
            Rc::new(RoundingStrictFlowLogCheckpointAdjuster { step: 0.25 }) as Rc<dyn StrictFlowLogCheckpointModifier>,
            Rc::new(MinimumStrictFlowLogCheckpointAdjuster { minimum: 49.7 }),
            Rc::new(CategoryStrictFlowLogCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.04),
                    ("south".to_string(), 1.22),
                    ("west".to_string(), 1.36),
                ]),
            }),
            Rc::new(StrictFlowLogCheckpointBurstOffsetAdjuster { weight: 1.14 }),
            Rc::new(FlaggedStrictFlowLogCheckpointAdjuster {
                flags: flags.clone(),
                flag: "strict-flow-log-checkpoint-adaptive".to_string(),
                percent: 4.6,
            }),
            Rc::new(CapStrictFlowLogCheckpointAdjuster { cap: 10989.0 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 17546.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_flow_log_checkpoint_engine(&flags::new_registry());
        let item = new_strict_flow_log_checkpoint("code-270", "title-178");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
