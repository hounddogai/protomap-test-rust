use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline group calculation.
pub trait TimelineGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineGroupAdjuster {
    pub minimum: f64,
}

impl TimelineGroupAdjuster for MinimumTimelineGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineGroupAdjuster {
    pub step: f64,
}

impl TimelineGroupAdjuster for RoundingTimelineGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineGroupAdjuster for TieredTimelineGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineGroupAdjuster {
    pub percent: f64,
}

impl TimelineGroupAdjuster for PercentTimelineGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroupCostOffsetAdjuster {
    pub weight: f64,
}

impl TimelineGroupAdjuster for TimelineGroupCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineGroup, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

/// Runs the timeline group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineGroupEngine {
    pub steps: Vec<Rc<dyn TimelineGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineGroupEngine {
    pub fn price(&mut self, item: &TimelineGroup) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[TimelineGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_timeline_group_engine() -> TimelineGroupEngine {
    TimelineGroupEngine {
        steps: vec![
            Rc::new(MinimumTimelineGroupAdjuster { minimum: 38.0 }) as Rc<dyn TimelineGroupAdjuster>,
            Rc::new(RoundingTimelineGroupAdjuster { step: 0.5 }),
            Rc::new(TieredTimelineGroupAdjuster {
                bounds: vec![977.0, 3874.0],
                rates: vec![0.89, 1.12],
            }),
            Rc::new(PercentTimelineGroupAdjuster { percent: -11.6 }),
            Rc::new(TimelineGroupCostOffsetAdjuster { weight: 0.63 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 13613.6,
    }
}
