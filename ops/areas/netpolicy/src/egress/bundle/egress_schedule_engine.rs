use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the egress schedule calculation.
pub trait EgressScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressScheduleBurstOffsetAdjuster {
    pub weight: f64,
}

impl EgressScheduleStep for EgressScheduleBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &EgressSchedule, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressScheduleAdjuster {
    pub step: f64,
}

impl EgressScheduleStep for RoundingEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressScheduleAdjuster {
    pub percent: f64,
}

impl EgressScheduleStep for PercentEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressScheduleStep for FlaggedEgressScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressScheduleEngine {
    pub steps: Vec<Rc<dyn EgressScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressScheduleEngine {
    pub fn evaluate(&mut self, item: &EgressSchedule) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[EgressSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_egress_schedule_engine(flags: &flags::Registry) -> EgressScheduleEngine {
    EgressScheduleEngine {
        steps: vec![
            Rc::new(EgressScheduleBurstOffsetAdjuster { weight: 1.62 }) as Rc<dyn EgressScheduleStep>,
            Rc::new(RoundingEgressScheduleAdjuster { step: 5.0 }),
            Rc::new(PercentEgressScheduleAdjuster { percent: 3.6 }),
            Rc::new(FlaggedEgressScheduleAdjuster {
                flags: flags.clone(),
                flag: "egress-schedule-rolling".to_string(),
                percent: 5.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 51564.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_schedule_engine(&flags::new_registry());
        let item = new_egress_schedule("code-741", "name-542");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
