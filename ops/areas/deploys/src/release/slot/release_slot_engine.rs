use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release slot calculation.
pub trait ReleaseSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseSlotAdjuster {
    pub step: f64,
}

impl ReleaseSlotStep for RoundingReleaseSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseSlotAdjuster {
    pub minimum: f64,
}

impl ReleaseSlotStep for MinimumReleaseSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseSlotAdjuster {
    pub percent: f64,
}

impl ReleaseSlotStep for PercentReleaseSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseSlotStep for TieredReleaseSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelReleaseSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseSlotStep for ChannelReleaseSlotAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ReleaseSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseSlotStep for FlaggedReleaseSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseSlotEngine {
    pub steps: Vec<Rc<dyn ReleaseSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseSlotEngine {
    pub fn evaluate(&mut self, item: &ReleaseSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ReleaseSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_release_slot_engine(flags: &flags::Registry) -> ReleaseSlotEngine {
    ReleaseSlotEngine {
        steps: vec![
            Rc::new(RoundingReleaseSlotAdjuster { step: 0.25 }) as Rc<dyn ReleaseSlotStep>,
            Rc::new(MinimumReleaseSlotAdjuster { minimum: 28.6 }),
            Rc::new(PercentReleaseSlotAdjuster { percent: -21.9 }),
            Rc::new(TieredReleaseSlotAdjuster {
                bounds: vec![610.0, 1777.0, 3901.0, 4726.0],
                rates: vec![0.83, 0.91, 0.84, 0.95],
            }),
            Rc::new(ChannelReleaseSlotAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.2), ("central".to_string(), 0.71)]),
            }),
            Rc::new(FlaggedReleaseSlotAdjuster {
                flags: flags.clone(),
                flag: "release-slot-capped".to_string(),
                percent: 10.0,
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 18553.5,
    }
}
