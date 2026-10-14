use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout notice calculation.
pub trait RolloutNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutNotice, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutNoticeFactor for FlaggedRolloutNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutNoticeAdjuster {
    pub cap: f64,
}

impl RolloutNoticeFactor for CapRolloutNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutNoticeDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl RolloutNoticeFactor for RolloutNoticeDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutNotice, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutNoticeFactor for TieredRolloutNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutNoticeAdjuster {
    pub percent: f64,
}

impl RolloutNoticeFactor for PercentRolloutNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollout notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutNoticeEngine {
    pub steps: Vec<Rc<dyn RolloutNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutNoticeEngine {
    pub fn compute(&mut self, item: &RolloutNotice) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[RolloutNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollout_notice_engine(flags: &flags::Registry) -> RolloutNoticeEngine {
    RolloutNoticeEngine {
        steps: vec![
            Rc::new(FlaggedRolloutNoticeAdjuster {
                flags: flags.clone(),
                flag: "rollout-notice-weighted".to_string(),
                percent: -6.6,
            }) as Rc<dyn RolloutNoticeFactor>,
            Rc::new(CapRolloutNoticeAdjuster { cap: 27052.0 }),
            Rc::new(RolloutNoticeDurationMinutesOffsetAdjuster { weight: 1.85 }),
            Rc::new(TieredRolloutNoticeAdjuster {
                bounds: vec![4369.0, 6096.0],
                rates: vec![1.13, 0.87],
            }),
            Rc::new(PercentRolloutNoticeAdjuster { percent: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 85528.5,
    }
}
