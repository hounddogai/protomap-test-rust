use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the approval checkpoint calculation.
pub trait ApprovalCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalCheckpointAdjuster {
    pub cap: f64,
}

impl ApprovalCheckpointFactor for CapApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalCheckpointFactor for FlaggedApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalCheckpointLimitOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalCheckpointFactor for ApprovalCheckpointLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalCheckpoint, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleApprovalCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalCheckpointFactor for LocaleApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &ApprovalCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalCheckpointAdjuster {
    pub percent: f64,
}

impl ApprovalCheckpointFactor for PercentApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalCheckpointAdjuster {
    pub minimum: f64,
}

impl ApprovalCheckpointFactor for MinimumApprovalCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the approval checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalCheckpointEngine {
    pub steps: Vec<Rc<dyn ApprovalCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalCheckpointEngine {
    pub fn evaluate(&mut self, item: &ApprovalCheckpoint) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalCheckpointFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_approval_checkpoint_engine(flags: &flags::Registry) -> ApprovalCheckpointEngine {
    ApprovalCheckpointEngine {
        steps: vec![
            Rc::new(CapApprovalCheckpointAdjuster { cap: 2761.0 }) as Rc<dyn ApprovalCheckpointFactor>,
            Rc::new(FlaggedApprovalCheckpointAdjuster {
                flags: flags.clone(),
                flag: "approval-checkpoint-primary".to_string(),
                percent: 5.8,
            }),
            Rc::new(ApprovalCheckpointLimitOffsetAdjuster { weight: 1.64 }),
            Rc::new(LocaleApprovalCheckpointAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.18), ("retail".to_string(), 0.96)]),
            }),
            Rc::new(PercentApprovalCheckpointAdjuster { percent: -4.8 }),
            Rc::new(MinimumApprovalCheckpointAdjuster { minimum: 48.4 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 57969.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_checkpoint_engine(&flags::new_registry());
        let item = new_approval_checkpoint("code-676", "title-902");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
