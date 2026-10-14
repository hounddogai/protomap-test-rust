use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled zone override calculation.
pub trait ScheduledZoneOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledZoneOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledZoneOverrideAdjuster {
    pub step: f64,
}

impl ScheduledZoneOverrideModifier for RoundingScheduledZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledZoneOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledZoneOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledZoneOverrideModifier for FlaggedScheduledZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledZoneOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledZoneOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledZoneOverrideModifier for TieredScheduledZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledZoneOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledZoneOverrideAdjuster {
    pub cap: f64,
}

impl ScheduledZoneOverrideModifier for CapScheduledZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledZoneOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled zone override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledZoneOverrideEngine {
    pub steps: Vec<Rc<dyn ScheduledZoneOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledZoneOverrideEngine {
    pub fn run(&mut self, item: &ScheduledZoneOverride) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledZoneOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_zone_override_engine(flags: &flags::Registry) -> ScheduledZoneOverrideEngine {
    ScheduledZoneOverrideEngine {
        steps: vec![
            Rc::new(RoundingScheduledZoneOverrideAdjuster { step: 0.1 }) as Rc<dyn ScheduledZoneOverrideModifier>,
            Rc::new(FlaggedScheduledZoneOverrideAdjuster {
                flags: flags.clone(),
                flag: "scheduled-zone-override-seasonal".to_string(),
                percent: 9.1,
            }),
            Rc::new(TieredScheduledZoneOverrideAdjuster {
                bounds: vec![2519.0, 8145.0],
                rates: vec![0.98, 0.92],
            }),
            Rc::new(CapScheduledZoneOverrideAdjuster { cap: 36269.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 48790.2,
    }
}
