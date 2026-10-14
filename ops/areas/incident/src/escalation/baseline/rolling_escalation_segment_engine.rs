use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling escalation segment calculation.
pub trait RollingEscalationSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingEscalationSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingEscalationSegmentAdjuster {
    pub minimum: f64,
}

impl RollingEscalationSegmentModifier for MinimumRollingEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingEscalationSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRollingEscalationSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingEscalationSegmentModifier for SourceRollingEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RollingEscalationSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingEscalationSegmentAdjuster {
    pub cap: f64,
}

impl RollingEscalationSegmentModifier for CapRollingEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingEscalationSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingEscalationSegmentAdjuster {
    pub step: f64,
}

impl RollingEscalationSegmentModifier for RoundingRollingEscalationSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingEscalationSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rolling escalation segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingEscalationSegmentEngine {
    pub steps: Vec<Rc<dyn RollingEscalationSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingEscalationSegmentEngine {
    pub fn run(&mut self, item: &RollingEscalationSegment) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingEscalationSegmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingEscalationSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_escalation_segment_engine() -> RollingEscalationSegmentEngine {
    RollingEscalationSegmentEngine {
        steps: vec![
            Rc::new(MinimumRollingEscalationSegmentAdjuster { minimum: 11.3 })
                as Rc<dyn RollingEscalationSegmentModifier>,
            Rc::new(SourceRollingEscalationSegmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.36), ("online".to_string(), 0.92)]),
            }),
            Rc::new(CapRollingEscalationSegmentAdjuster { cap: 16894.0 }),
            Rc::new(RoundingRollingEscalationSegmentAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 94993.7,
    }
}
