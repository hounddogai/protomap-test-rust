use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist schedule calculation.
pub trait AllowlistScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAllowlistScheduleAdjuster {
    pub percent: f64,
}

impl AllowlistScheduleFactor for PercentAllowlistScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AllowlistSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAllowlistScheduleAdjuster {
    pub step: f64,
}

impl AllowlistScheduleFactor for RoundingAllowlistScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AllowlistSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistScheduleBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistScheduleFactor for AllowlistScheduleBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistSchedule, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistScheduleFactor for FlaggedAllowlistScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the allowlist schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistScheduleEngine {
    pub steps: Vec<Rc<dyn AllowlistScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistScheduleEngine {
    pub fn evaluate(&mut self, item: &AllowlistSchedule) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_allowlist_schedule_engine(flags: &flags::Registry) -> AllowlistScheduleEngine {
    AllowlistScheduleEngine {
        steps: vec![
            Rc::new(PercentAllowlistScheduleAdjuster { percent: -2.2 }) as Rc<dyn AllowlistScheduleFactor>,
            Rc::new(RoundingAllowlistScheduleAdjuster { step: 0.1 }),
            Rc::new(AllowlistScheduleBatchSizeOffsetAdjuster { weight: 0.45 }),
            Rc::new(FlaggedAllowlistScheduleAdjuster {
                flags: flags.clone(),
                flag: "allowlist-schedule-weighted".to_string(),
                percent: -4.0,
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 63749.0,
    }
}
