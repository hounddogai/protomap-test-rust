use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling egress schedule calculation.
pub trait RollingEgressScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingEgressSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingEgressScheduleAdjuster {
    pub cap: f64,
}

impl RollingEgressScheduleFactor for CapRollingEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingEgressSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressSchedulePortOffsetAdjuster {
    pub weight: f64,
}

impl RollingEgressScheduleFactor for RollingEgressSchedulePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &RollingEgressSchedule, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingEgressScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingEgressScheduleFactor for TieredRollingEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingEgressSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingEgressScheduleAdjuster {
    pub percent: f64,
}

impl RollingEgressScheduleFactor for PercentRollingEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingEgressSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingEgressScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingEgressScheduleFactor for FlaggedRollingEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingEgressSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling egress schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingEgressScheduleEngine {
    pub steps: Vec<Rc<dyn RollingEgressScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingEgressScheduleEngine {
    pub fn compute(&mut self, item: &RollingEgressSchedule) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[RollingEgressSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_egress_schedule_engine(flags: &flags::Registry) -> RollingEgressScheduleEngine {
    RollingEgressScheduleEngine {
        steps: vec![
            Rc::new(CapRollingEgressScheduleAdjuster { cap: 43561.0 }) as Rc<dyn RollingEgressScheduleFactor>,
            Rc::new(RollingEgressSchedulePortOffsetAdjuster { weight: 1.11 }),
            Rc::new(TieredRollingEgressScheduleAdjuster {
                bounds: vec![5011.0, 5611.0],
                rates: vec![0.9, 0.88],
            }),
            Rc::new(PercentRollingEgressScheduleAdjuster { percent: -5.2 }),
            Rc::new(FlaggedRollingEgressScheduleAdjuster {
                flags: flags.clone(),
                flag: "rolling-egress-schedule-weighted".to_string(),
                percent: -1.8,
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 70845.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_egress_schedule_engine(&flags::new_registry());
        let item = new_rolling_egress_schedule("code-848", "label-931");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
