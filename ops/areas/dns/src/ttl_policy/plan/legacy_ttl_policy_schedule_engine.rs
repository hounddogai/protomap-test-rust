use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy ttl policy schedule calculation.
pub trait LegacyTtlPolicyScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyTtlPolicySchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyTtlPolicyScheduleAdjuster {
    pub step: f64,
}

impl LegacyTtlPolicyScheduleFactor for RoundingLegacyTtlPolicyScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyTtlPolicySchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyTtlPolicyScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyTtlPolicyScheduleFactor for TieredLegacyTtlPolicyScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyTtlPolicySchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyTtlPolicyScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyTtlPolicyScheduleFactor for FlaggedLegacyTtlPolicyScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyTtlPolicySchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicyScheduleCeilingOffsetAdjuster {
    pub weight: f64,
}

impl LegacyTtlPolicyScheduleFactor for LegacyTtlPolicyScheduleCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyTtlPolicySchedule, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyTtlPolicyScheduleAdjuster {
    pub percent: f64,
}

impl LegacyTtlPolicyScheduleFactor for PercentLegacyTtlPolicyScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyTtlPolicySchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy ttl policy schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyTtlPolicyScheduleEngine {
    pub steps: Vec<Rc<dyn LegacyTtlPolicyScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyTtlPolicyScheduleEngine {
    pub fn evaluate(&mut self, item: &LegacyTtlPolicySchedule) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyTtlPolicySchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_ttl_policy_schedule_engine(flags: &flags::Registry) -> LegacyTtlPolicyScheduleEngine {
    LegacyTtlPolicyScheduleEngine {
        steps: vec![
            Rc::new(RoundingLegacyTtlPolicyScheduleAdjuster { step: 0.05 }) as Rc<dyn LegacyTtlPolicyScheduleFactor>,
            Rc::new(TieredLegacyTtlPolicyScheduleAdjuster {
                bounds: vec![5669.0, 5699.0, 6193.0],
                rates: vec![1.01, 1.0, 1.14],
            }),
            Rc::new(FlaggedLegacyTtlPolicyScheduleAdjuster {
                flags: flags.clone(),
                flag: "legacy-ttl-policy-schedule-scheduled".to_string(),
                percent: -5.0,
            }),
            Rc::new(LegacyTtlPolicyScheduleCeilingOffsetAdjuster { weight: 1.08 }),
            Rc::new(PercentLegacyTtlPolicyScheduleAdjuster { percent: -17.3 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 28641.5,
    }
}
