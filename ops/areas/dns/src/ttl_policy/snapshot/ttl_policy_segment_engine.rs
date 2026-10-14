use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the ttl policy segment calculation.
pub trait TtlPolicySegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicySegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicySegmentAdjuster {
    pub step: f64,
}

impl TtlPolicySegmentAdjuster for RoundingTtlPolicySegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicySegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTtlPolicySegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicySegmentAdjuster for NameTtlPolicySegmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TtlPolicySegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicySegmentAdjuster {
    pub minimum: f64,
}

impl TtlPolicySegmentAdjuster for MinimumTtlPolicySegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicySegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicySegmentAdjuster {
    pub cap: f64,
}

impl TtlPolicySegmentAdjuster for CapTtlPolicySegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicySegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the ttl policy segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicySegmentEngine {
    pub steps: Vec<Rc<dyn TtlPolicySegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicySegmentEngine {
    pub fn run(&mut self, item: &TtlPolicySegment) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicySegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_segment_engine() -> TtlPolicySegmentEngine {
    TtlPolicySegmentEngine {
        steps: vec![
            Rc::new(RoundingTtlPolicySegmentAdjuster { step: 0.05 }) as Rc<dyn TtlPolicySegmentAdjuster>,
            Rc::new(NameTtlPolicySegmentAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.81),
                    ("west".to_string(), 0.81),
                    ("north".to_string(), 1.21),
                ]),
            }),
            Rc::new(MinimumTtlPolicySegmentAdjuster { minimum: 40.6 }),
            Rc::new(CapTtlPolicySegmentAdjuster { cap: 5711.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 6834.8,
    }
}
