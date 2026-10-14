use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline snapshot calculation.
pub trait TimelineSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineSnapshotFactor for TieredTimelineSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineSnapshotAdjuster {
    pub minimum: f64,
}

impl TimelineSnapshotFactor for MinimumTimelineSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineSnapshotAdjuster {
    pub step: f64,
}

impl TimelineSnapshotFactor for RoundingTimelineSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the timeline snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineSnapshotEngine {
    pub steps: Vec<Rc<dyn TimelineSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineSnapshotEngine {
    pub fn evaluate(&mut self, item: &TimelineSnapshot) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[TimelineSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_timeline_snapshot_engine() -> TimelineSnapshotEngine {
    TimelineSnapshotEngine {
        steps: vec![
            Rc::new(TieredTimelineSnapshotAdjuster {
                bounds: vec![4079.0, 7006.0, 9663.0],
                rates: vec![0.95, 0.85, 1.12],
            }) as Rc<dyn TimelineSnapshotFactor>,
            Rc::new(MinimumTimelineSnapshotAdjuster { minimum: 5.7 }),
            Rc::new(RoundingTimelineSnapshotAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 88185.1,
    }
}
