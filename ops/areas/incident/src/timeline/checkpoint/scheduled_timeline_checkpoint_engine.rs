use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled timeline checkpoint calculation.
pub trait ScheduledTimelineCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledTimelineCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledTimelineCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledTimelineCheckpointAdjuster for FlaggedScheduledTimelineCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledTimelineCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledTimelineCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledTimelineCheckpointAdjuster for TieredScheduledTimelineCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledTimelineCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledTimelineCheckpointAdjuster {
    pub percent: f64,
}

impl ScheduledTimelineCheckpointAdjuster for PercentScheduledTimelineCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledTimelineCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled timeline checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledTimelineCheckpointEngine {
    pub steps: Vec<Rc<dyn ScheduledTimelineCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledTimelineCheckpointEngine {
    pub fn compute(&mut self, item: &ScheduledTimelineCheckpoint) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledTimelineCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_timeline_checkpoint_engine(flags: &flags::Registry) -> ScheduledTimelineCheckpointEngine {
    ScheduledTimelineCheckpointEngine {
        steps: vec![
            Rc::new(FlaggedScheduledTimelineCheckpointAdjuster {
                flags: flags.clone(),
                flag: "scheduled-timeline-checkpoint-lenient".to_string(),
                percent: -3.0,
            }) as Rc<dyn ScheduledTimelineCheckpointAdjuster>,
            Rc::new(TieredScheduledTimelineCheckpointAdjuster {
                bounds: vec![3409.0, 3459.0, 7486.0],
                rates: vec![0.81, 1.0, 1.17],
            }),
            Rc::new(PercentScheduledTimelineCheckpointAdjuster { percent: 9.8 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 90191.3,
    }
}
