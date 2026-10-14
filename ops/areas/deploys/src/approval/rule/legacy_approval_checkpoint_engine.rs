use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy approval checkpoint calculation.
pub trait LegacyApprovalCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyApprovalCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyApprovalCheckpointAdjuster {
    pub percent: f64,
}

impl LegacyApprovalCheckpointFactor for PercentLegacyApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyApprovalCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyApprovalCheckpointAdjuster {
    pub cap: f64,
}

impl LegacyApprovalCheckpointFactor for CapLegacyApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyApprovalCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyApprovalCheckpointCostOffsetAdjuster {
    pub weight: f64,
}

impl LegacyApprovalCheckpointFactor for LegacyApprovalCheckpointCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyApprovalCheckpoint, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyApprovalCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyApprovalCheckpointFactor for FlaggedLegacyApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyApprovalCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy approval checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyApprovalCheckpointEngine {
    pub steps: Vec<Rc<dyn LegacyApprovalCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyApprovalCheckpointEngine {
    pub fn evaluate(&mut self, item: &LegacyApprovalCheckpoint) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyApprovalCheckpointFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyApprovalCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_approval_checkpoint_engine(flags: &flags::Registry) -> LegacyApprovalCheckpointEngine {
    LegacyApprovalCheckpointEngine {
        steps: vec![
            Rc::new(PercentLegacyApprovalCheckpointAdjuster { percent: -20.1 })
                as Rc<dyn LegacyApprovalCheckpointFactor>,
            Rc::new(CapLegacyApprovalCheckpointAdjuster { cap: 30124.0 }),
            Rc::new(LegacyApprovalCheckpointCostOffsetAdjuster { weight: 0.42 }),
            Rc::new(FlaggedLegacyApprovalCheckpointAdjuster {
                flags: flags.clone(),
                flag: "legacy-approval-checkpoint-default".to_string(),
                percent: -4.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 60617.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_approval_checkpoint_engine(&flags::new_registry());
        let item = new_legacy_approval_checkpoint("code-734", "name-892");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
