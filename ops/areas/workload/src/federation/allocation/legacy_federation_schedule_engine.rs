use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy federation schedule calculation.
pub trait LegacyFederationScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFederationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyFederationScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyFederationScheduleAdjuster for FlaggedLegacyFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyFederationSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyFederationScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyFederationScheduleAdjuster for TieredLegacyFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyFederationSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationScheduleVersionOffsetAdjuster {
    pub weight: f64,
}

impl LegacyFederationScheduleAdjuster for LegacyFederationScheduleVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyFederationSchedule, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyFederationScheduleAdjuster {
    pub percent: f64,
}

impl LegacyFederationScheduleAdjuster for PercentLegacyFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyFederationSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy federation schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFederationScheduleEngine {
    pub steps: Vec<Rc<dyn LegacyFederationScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFederationScheduleEngine {
    pub fn price(&mut self, item: &LegacyFederationSchedule) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFederationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_federation_schedule_engine(flags: &flags::Registry) -> LegacyFederationScheduleEngine {
    LegacyFederationScheduleEngine {
        steps: vec![
            Rc::new(FlaggedLegacyFederationScheduleAdjuster {
                flags: flags.clone(),
                flag: "legacy-federation-schedule-strict".to_string(),
                percent: -0.2,
            }) as Rc<dyn LegacyFederationScheduleAdjuster>,
            Rc::new(TieredLegacyFederationScheduleAdjuster {
                bounds: vec![8355.0, 9536.0],
                rates: vec![1.17, 1.14],
            }),
            Rc::new(LegacyFederationScheduleVersionOffsetAdjuster { weight: 1.12 }),
            Rc::new(PercentLegacyFederationScheduleAdjuster { percent: -18.0 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 33196.2,
    }
}
