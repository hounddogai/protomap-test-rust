use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline claim calculation.
pub trait TimelineClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineClaimAdjuster {
    pub step: f64,
}

impl TimelineClaimModifier for RoundingTimelineClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineClaimAdjuster {
    pub percent: f64,
}

impl TimelineClaimModifier for PercentTimelineClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTimelineClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TimelineClaimModifier for SourceTimelineClaimAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TimelineClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the timeline claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineClaimEngine {
    pub steps: Vec<Rc<dyn TimelineClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineClaimEngine {
    pub fn evaluate(&mut self, item: &TimelineClaim) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[TimelineClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_timeline_claim_engine() -> TimelineClaimEngine {
    TimelineClaimEngine {
        steps: vec![
            Rc::new(RoundingTimelineClaimAdjuster { step: 0.5 }) as Rc<dyn TimelineClaimModifier>,
            Rc::new(PercentTimelineClaimAdjuster { percent: -8.4 }),
            Rc::new(SourceTimelineClaimAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.21), ("east".to_string(), 0.91)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 64791.7,
    }
}
