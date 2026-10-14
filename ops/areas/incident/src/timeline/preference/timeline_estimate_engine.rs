use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the timeline estimate calculation.
pub trait TimelineEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineEstimateAdjuster {
    pub cap: f64,
}

impl TimelineEstimateModifier for CapTimelineEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineEstimateAdjuster {
    pub minimum: f64,
}

impl TimelineEstimateModifier for MinimumTimelineEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineEstimateAdjuster {
    pub step: f64,
}

impl TimelineEstimateModifier for RoundingTimelineEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTimelineEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TimelineEstimateModifier for FlaggedTimelineEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TimelineEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the timeline estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineEstimateEngine {
    pub steps: Vec<Rc<dyn TimelineEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineEstimateEngine {
    pub fn run(&mut self, item: &TimelineEstimate) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TimelineEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_timeline_estimate_engine(flags: &flags::Registry) -> TimelineEstimateEngine {
    TimelineEstimateEngine {
        steps: vec![
            Rc::new(CapTimelineEstimateAdjuster { cap: 30589.0 }) as Rc<dyn TimelineEstimateModifier>,
            Rc::new(MinimumTimelineEstimateAdjuster { minimum: 7.3 }),
            Rc::new(RoundingTimelineEstimateAdjuster { step: 5.0 }),
            Rc::new(FlaggedTimelineEstimateAdjuster {
                flags: flags.clone(),
                flag: "timeline-estimate-seasonal".to_string(),
                percent: 2.3,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 85799.5,
    }
}
