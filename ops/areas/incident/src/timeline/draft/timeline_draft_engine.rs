use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline draft calculation.
pub trait TimelineDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineDraftAdjuster {
    pub cap: f64,
}

impl TimelineDraftStep for CapTimelineDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineDraftAdjuster {
    pub percent: f64,
}

impl TimelineDraftStep for PercentTimelineDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTimelineDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TimelineDraftStep for NameTimelineDraftAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TimelineDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineDraftDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl TimelineDraftStep for TimelineDraftDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineDraft, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineDraftAdjuster {
    pub step: f64,
}

impl TimelineDraftStep for RoundingTimelineDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the timeline draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineDraftEngine {
    pub steps: Vec<Rc<dyn TimelineDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineDraftEngine {
    pub fn compute(&mut self, item: &TimelineDraft) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TimelineDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TimelineDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_timeline_draft_engine() -> TimelineDraftEngine {
    TimelineDraftEngine {
        steps: vec![
            Rc::new(CapTimelineDraftAdjuster { cap: 15589.0 }) as Rc<dyn TimelineDraftStep>,
            Rc::new(PercentTimelineDraftAdjuster { percent: 3.0 }),
            Rc::new(NameTimelineDraftAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.97),
                    ("retail".to_string(), 0.74),
                    ("online".to_string(), 0.73),
                ]),
            }),
            Rc::new(TimelineDraftDurationMinutesOffsetAdjuster { weight: 1.55 }),
            Rc::new(RoundingTimelineDraftAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 30979.1,
    }
}
