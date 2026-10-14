use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout segment calculation.
pub trait RolloutSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutSegmentAdjuster {
    pub percent: f64,
}

impl RolloutSegmentStep for PercentRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutSegmentAdjuster {
    pub cap: f64,
}

impl RolloutSegmentStep for CapRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutSegmentStep for FlaggedRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutSegmentAdjuster {
    pub minimum: f64,
}

impl RolloutSegmentStep for MinimumRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rollout segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutSegmentEngine {
    pub steps: Vec<Rc<dyn RolloutSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutSegmentEngine {
    pub fn evaluate(&mut self, item: &RolloutSegment) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[RolloutSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_segment_engine(flags: &flags::Registry) -> RolloutSegmentEngine {
    RolloutSegmentEngine {
        steps: vec![
            Rc::new(PercentRolloutSegmentAdjuster { percent: -11.1 }) as Rc<dyn RolloutSegmentStep>,
            Rc::new(CapRolloutSegmentAdjuster { cap: 16742.0 }),
            Rc::new(FlaggedRolloutSegmentAdjuster {
                flags: flags.clone(),
                flag: "rollout-segment-global".to_string(),
                percent: 7.9,
            }),
            Rc::new(MinimumRolloutSegmentAdjuster { minimum: 24.4 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 42569.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_segment_engine(&flags::new_registry());
        let item = new_rollout_segment("code-749", "title-690");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
