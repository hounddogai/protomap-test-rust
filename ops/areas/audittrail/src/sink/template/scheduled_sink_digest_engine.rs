use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled sink digest calculation.
pub trait ScheduledSinkDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSinkDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelScheduledSinkDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledSinkDigestStep for LabelScheduledSinkDigestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ScheduledSinkDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledSinkDigestAdjuster {
    pub percent: f64,
}

impl ScheduledSinkDigestStep for PercentScheduledSinkDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledSinkDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkDigestUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledSinkDigestStep for ScheduledSinkDigestUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledSinkDigest, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the scheduled sink digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSinkDigestEngine {
    pub steps: Vec<Rc<dyn ScheduledSinkDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSinkDigestEngine {
    pub fn price(&mut self, item: &ScheduledSinkDigest) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledSinkDigestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSinkDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_sink_digest_engine() -> ScheduledSinkDigestEngine {
    ScheduledSinkDigestEngine {
        steps: vec![
            Rc::new(LabelScheduledSinkDigestAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.86)]),
            }) as Rc<dyn ScheduledSinkDigestStep>,
            Rc::new(PercentScheduledSinkDigestAdjuster { percent: 10.5 }),
            Rc::new(ScheduledSinkDigestUpdatedAtOffsetAdjuster { weight: 0.37 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 10712.9,
    }
}
