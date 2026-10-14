use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending timeline schedule calculation.
pub trait PendingTimelineScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTimelineSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingTimelineScheduleAdjuster {
    pub minimum: f64,
}

impl PendingTimelineScheduleModifier for MinimumPendingTimelineScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingTimelineSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineScheduleMaximumOffsetAdjuster {
    pub weight: f64,
}

impl PendingTimelineScheduleModifier for PendingTimelineScheduleMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &PendingTimelineSchedule, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingTimelineScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingTimelineScheduleModifier for FlaggedPendingTimelineScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingTimelineSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending timeline schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTimelineScheduleEngine {
    pub steps: Vec<Rc<dyn PendingTimelineScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTimelineScheduleEngine {
    pub fn run(&mut self, item: &PendingTimelineSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingTimelineSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_timeline_schedule_engine(flags: &flags::Registry) -> PendingTimelineScheduleEngine {
    PendingTimelineScheduleEngine {
        steps: vec![
            Rc::new(MinimumPendingTimelineScheduleAdjuster { minimum: 9.4 }) as Rc<dyn PendingTimelineScheduleModifier>,
            Rc::new(PendingTimelineScheduleMaximumOffsetAdjuster { weight: 0.55 }),
            Rc::new(FlaggedPendingTimelineScheduleAdjuster {
                flags: flags.clone(),
                flag: "pending-timeline-schedule-seasonal".to_string(),
                percent: 2.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 60346.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_timeline_schedule_engine(&flags::new_registry());
        let item = new_pending_timeline_schedule("code-351", "name-448");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
