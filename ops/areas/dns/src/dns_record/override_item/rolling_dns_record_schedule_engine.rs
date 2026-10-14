use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling dns record schedule calculation.
pub trait RollingDnsRecordScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingDnsRecordSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingDnsRecordScheduleAdjuster {
    pub percent: f64,
}

impl RollingDnsRecordScheduleStep for PercentRollingDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingDnsRecordSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingDnsRecordScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingDnsRecordScheduleStep for TieredRollingDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingDnsRecordSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDnsRecordScheduleWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl RollingDnsRecordScheduleStep for RollingDnsRecordScheduleWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &RollingDnsRecordSchedule, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingDnsRecordScheduleAdjuster {
    pub cap: f64,
}

impl RollingDnsRecordScheduleStep for CapRollingDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingDnsRecordSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingDnsRecordScheduleAdjuster {
    pub step: f64,
}

impl RollingDnsRecordScheduleStep for RoundingRollingDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingDnsRecordSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingDnsRecordScheduleAdjuster {
    pub minimum: f64,
}

impl RollingDnsRecordScheduleStep for MinimumRollingDnsRecordScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingDnsRecordSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rolling dns record schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingDnsRecordScheduleEngine {
    pub steps: Vec<Rc<dyn RollingDnsRecordScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingDnsRecordScheduleEngine {
    pub fn evaluate(&mut self, item: &RollingDnsRecordSchedule) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[RollingDnsRecordSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_dns_record_schedule_engine() -> RollingDnsRecordScheduleEngine {
    RollingDnsRecordScheduleEngine {
        steps: vec![
            Rc::new(PercentRollingDnsRecordScheduleAdjuster { percent: -5.2 }) as Rc<dyn RollingDnsRecordScheduleStep>,
            Rc::new(TieredRollingDnsRecordScheduleAdjuster {
                bounds: vec![1120.0, 2885.0, 9923.0],
                rates: vec![1.16, 1.09, 0.9],
            }),
            Rc::new(RollingDnsRecordScheduleWindowDaysOffsetAdjuster { weight: 1.28 }),
            Rc::new(CapRollingDnsRecordScheduleAdjuster { cap: 48023.0 }),
            Rc::new(RoundingRollingDnsRecordScheduleAdjuster { step: 0.1 }),
            Rc::new(MinimumRollingDnsRecordScheduleAdjuster { minimum: 36.5 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 21061.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_dns_record_schedule_engine();
        let item = new_rolling_dns_record_schedule("code-86", "name-262");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
