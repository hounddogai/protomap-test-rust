use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled action item segment calculation.
pub trait ScheduledActionItemSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledActionItemSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledActionItemSegmentAdjuster {
    pub step: f64,
}

impl ScheduledActionItemSegmentFactor for RoundingScheduledActionItemSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledActionItemSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledActionItemSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledActionItemSegmentFactor for TieredScheduledActionItemSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledActionItemSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryScheduledActionItemSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledActionItemSegmentFactor for CountryScheduledActionItemSegmentAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ScheduledActionItemSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledActionItemSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledActionItemSegmentFactor for FlaggedScheduledActionItemSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledActionItemSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledActionItemSegmentAdjuster {
    pub cap: f64,
}

impl ScheduledActionItemSegmentFactor for CapScheduledActionItemSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledActionItemSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled action item segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledActionItemSegmentEngine {
    pub steps: Vec<Rc<dyn ScheduledActionItemSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledActionItemSegmentEngine {
    pub fn evaluate(&mut self, item: &ScheduledActionItemSegment) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledActionItemSegmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledActionItemSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_action_item_segment_engine(flags: &flags::Registry) -> ScheduledActionItemSegmentEngine {
    ScheduledActionItemSegmentEngine {
        steps: vec![
            Rc::new(RoundingScheduledActionItemSegmentAdjuster { step: 0.1 })
                as Rc<dyn ScheduledActionItemSegmentFactor>,
            Rc::new(TieredScheduledActionItemSegmentAdjuster {
                bounds: vec![682.0, 6460.0],
                rates: vec![1.08, 0.99],
            }),
            Rc::new(CountryScheduledActionItemSegmentAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.37)]),
            }),
            Rc::new(FlaggedScheduledActionItemSegmentAdjuster {
                flags: flags.clone(),
                flag: "scheduled-action-item-segment-adaptive".to_string(),
                percent: -8.7,
            }),
            Rc::new(CapScheduledActionItemSegmentAdjuster { cap: 18774.0 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 75644.7,
    }
}
