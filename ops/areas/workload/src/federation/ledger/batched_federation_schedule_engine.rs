use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched federation schedule calculation.
pub trait BatchedFederationScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedFederationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedFederationScheduleAdjuster {
    pub cap: f64,
}

impl BatchedFederationScheduleStep for CapBatchedFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedFederationSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedFederationScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedFederationScheduleStep for FlaggedBatchedFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedFederationSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationScheduleCostOffsetAdjuster {
    pub weight: f64,
}

impl BatchedFederationScheduleStep for BatchedFederationScheduleCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedFederationSchedule, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

/// Runs the batched federation schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedFederationScheduleEngine {
    pub steps: Vec<Rc<dyn BatchedFederationScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedFederationScheduleEngine {
    pub fn run(&mut self, item: &BatchedFederationSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedFederationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_federation_schedule_engine(flags: &flags::Registry) -> BatchedFederationScheduleEngine {
    BatchedFederationScheduleEngine {
        steps: vec![
            Rc::new(CapBatchedFederationScheduleAdjuster { cap: 48389.0 }) as Rc<dyn BatchedFederationScheduleStep>,
            Rc::new(FlaggedBatchedFederationScheduleAdjuster {
                flags: flags.clone(),
                flag: "batched-federation-schedule-scheduled".to_string(),
                percent: -7.3,
            }),
            Rc::new(BatchedFederationScheduleCostOffsetAdjuster { weight: 1.36 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 91362.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_federation_schedule_engine(&flags::new_registry());
        let item = new_batched_federation_schedule("code-196", "name-844");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
