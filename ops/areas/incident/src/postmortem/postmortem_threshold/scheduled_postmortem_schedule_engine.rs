use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled postmortem schedule calculation.
pub trait ScheduledPostmortemScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledPostmortemSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemSchedulePortOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledPostmortemScheduleStep for ScheduledPostmortemSchedulePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledPostmortemSchedule, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledPostmortemScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledPostmortemScheduleStep for TieredScheduledPostmortemScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledPostmortemSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledPostmortemScheduleAdjuster {
    pub minimum: f64,
}

impl ScheduledPostmortemScheduleStep for MinimumScheduledPostmortemScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledPostmortemSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled postmortem schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledPostmortemScheduleEngine {
    pub steps: Vec<Rc<dyn ScheduledPostmortemScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledPostmortemScheduleEngine {
    pub fn evaluate(&mut self, item: &ScheduledPostmortemSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledPostmortemSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_postmortem_schedule_engine() -> ScheduledPostmortemScheduleEngine {
    ScheduledPostmortemScheduleEngine {
        steps: vec![
            Rc::new(ScheduledPostmortemSchedulePortOffsetAdjuster { weight: 0.14 })
                as Rc<dyn ScheduledPostmortemScheduleStep>,
            Rc::new(TieredScheduledPostmortemScheduleAdjuster {
                bounds: vec![1732.0, 4053.0, 4904.0],
                rates: vec![0.96, 0.82, 0.89],
            }),
            Rc::new(MinimumScheduledPostmortemScheduleAdjuster { minimum: 29.8 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 67257.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_postmortem_schedule_engine();
        let item = new_scheduled_postmortem_schedule("code-396", "title-170");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
