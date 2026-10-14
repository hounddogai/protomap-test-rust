use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline allocation calculation.
pub trait TimelineAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineAllocationAdjuster {
    pub percent: f64,
}

impl TimelineAllocationModifier for PercentTimelineAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineAllocationAdjuster {
    pub step: f64,
}

impl TimelineAllocationModifier for RoundingTimelineAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineAllocationAdjuster {
    pub cap: f64,
}

impl TimelineAllocationModifier for CapTimelineAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryTimelineAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TimelineAllocationModifier for CountryTimelineAllocationAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &TimelineAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the timeline allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineAllocationEngine {
    pub steps: Vec<Rc<dyn TimelineAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineAllocationEngine {
    pub fn run(&mut self, item: &TimelineAllocation) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[TimelineAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_timeline_allocation_engine() -> TimelineAllocationEngine {
    TimelineAllocationEngine {
        steps: vec![
            Rc::new(PercentTimelineAllocationAdjuster { percent: -0.9 }) as Rc<dyn TimelineAllocationModifier>,
            Rc::new(RoundingTimelineAllocationAdjuster { step: 5.0 }),
            Rc::new(CapTimelineAllocationAdjuster { cap: 12071.0 }),
            Rc::new(CountryTimelineAllocationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.94), ("east".to_string(), 0.9)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 21130.4,
    }
}
