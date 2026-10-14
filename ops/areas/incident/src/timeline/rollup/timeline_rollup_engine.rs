use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the timeline rollup calculation.
pub trait TimelineRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineRollupAdjuster for TieredTimelineRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineRollupAdjuster {
    pub cap: f64,
}

impl TimelineRollupAdjuster for CapTimelineRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRollupSequenceOffsetAdjuster {
    pub weight: f64,
}

impl TimelineRollupAdjuster for TimelineRollupSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineRollup, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineRollupAdjuster {
    pub percent: f64,
}

impl TimelineRollupAdjuster for PercentTimelineRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineRollupAdjuster {
    pub minimum: f64,
}

impl TimelineRollupAdjuster for MinimumTimelineRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the timeline rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineRollupEngine {
    pub steps: Vec<Rc<dyn TimelineRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineRollupEngine {
    pub fn run(&mut self, item: &TimelineRollup) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TimelineRollupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TimelineRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_timeline_rollup_engine() -> TimelineRollupEngine {
    TimelineRollupEngine {
        steps: vec![
            Rc::new(TieredTimelineRollupAdjuster {
                bounds: vec![1237.0, 4583.0, 4933.0, 6770.0],
                rates: vec![1.09, 0.87, 0.8, 0.89],
            }) as Rc<dyn TimelineRollupAdjuster>,
            Rc::new(CapTimelineRollupAdjuster { cap: 2263.0 }),
            Rc::new(TimelineRollupSequenceOffsetAdjuster { weight: 0.28 }),
            Rc::new(PercentTimelineRollupAdjuster { percent: -16.3 }),
            Rc::new(MinimumTimelineRollupAdjuster { minimum: 48.8 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 43467.8,
    }
}
