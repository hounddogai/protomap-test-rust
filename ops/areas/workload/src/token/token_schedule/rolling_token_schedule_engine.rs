use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling token schedule calculation.
pub trait RollingTokenScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTokenSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingTokenScheduleAdjuster {
    pub step: f64,
}

impl RollingTokenScheduleStep for RoundingRollingTokenScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingTokenSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTokenScheduleAdjuster {
    pub percent: f64,
}

impl RollingTokenScheduleStep for PercentRollingTokenScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTokenSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingTokenScheduleAdjuster {
    pub cap: f64,
}

impl RollingTokenScheduleStep for CapRollingTokenScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingTokenSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRollingTokenScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingTokenScheduleStep for UnitRollingTokenScheduleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RollingTokenSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingTokenScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingTokenScheduleStep for FlaggedRollingTokenScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingTokenSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling token schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTokenScheduleEngine {
    pub steps: Vec<Rc<dyn RollingTokenScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTokenScheduleEngine {
    pub fn compute(&mut self, item: &RollingTokenSchedule) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[RollingTokenSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_token_schedule_engine(flags: &flags::Registry) -> RollingTokenScheduleEngine {
    RollingTokenScheduleEngine {
        steps: vec![
            Rc::new(RoundingRollingTokenScheduleAdjuster { step: 0.5 }) as Rc<dyn RollingTokenScheduleStep>,
            Rc::new(PercentRollingTokenScheduleAdjuster { percent: -7.5 }),
            Rc::new(CapRollingTokenScheduleAdjuster { cap: 29001.0 }),
            Rc::new(UnitRollingTokenScheduleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.77), ("central".to_string(), 1.3)]),
            }),
            Rc::new(FlaggedRollingTokenScheduleAdjuster {
                flags: flags.clone(),
                flag: "rolling-token-schedule-manual".to_string(),
                percent: 2.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 22017.5,
    }
}
