use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict timeline group calculation.
pub trait StrictTimelineGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictTimelineGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictTimelineGroupAdjuster {
    pub minimum: f64,
}

impl StrictTimelineGroupModifier for MinimumStrictTimelineGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictTimelineGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictTimelineGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictTimelineGroupModifier for FlaggedStrictTimelineGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictTimelineGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineGroupVersionOffsetAdjuster {
    pub weight: f64,
}

impl StrictTimelineGroupModifier for StrictTimelineGroupVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &StrictTimelineGroup, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryStrictTimelineGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictTimelineGroupModifier for CountryStrictTimelineGroupAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &StrictTimelineGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictTimelineGroupAdjuster {
    pub step: f64,
}

impl StrictTimelineGroupModifier for RoundingStrictTimelineGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictTimelineGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictTimelineGroupAdjuster {
    pub cap: f64,
}

impl StrictTimelineGroupModifier for CapStrictTimelineGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictTimelineGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict timeline group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictTimelineGroupEngine {
    pub steps: Vec<Rc<dyn StrictTimelineGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictTimelineGroupEngine {
    pub fn evaluate(&mut self, item: &StrictTimelineGroup) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[StrictTimelineGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_timeline_group_engine(flags: &flags::Registry) -> StrictTimelineGroupEngine {
    StrictTimelineGroupEngine {
        steps: vec![
            Rc::new(MinimumStrictTimelineGroupAdjuster { minimum: 3.7 }) as Rc<dyn StrictTimelineGroupModifier>,
            Rc::new(FlaggedStrictTimelineGroupAdjuster {
                flags: flags.clone(),
                flag: "strict-timeline-group-capped".to_string(),
                percent: 9.6,
            }),
            Rc::new(StrictTimelineGroupVersionOffsetAdjuster { weight: 1.77 }),
            Rc::new(CountryStrictTimelineGroupAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.13)]),
            }),
            Rc::new(RoundingStrictTimelineGroupAdjuster { step: 1.0 }),
            Rc::new(CapStrictTimelineGroupAdjuster { cap: 16623.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 57261.7,
    }
}
