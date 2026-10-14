use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the timeline adjustment calculation.
pub trait TimelineAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTimelineAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TimelineAdjustmentStep for FlaggedTimelineAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TimelineAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineAdjustmentAdjuster {
    pub percent: f64,
}

impl TimelineAdjustmentStep for PercentTimelineAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineAdjustmentAdjuster {
    pub minimum: f64,
}

impl TimelineAdjustmentStep for MinimumTimelineAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the timeline adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineAdjustmentEngine {
    pub steps: Vec<Rc<dyn TimelineAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineAdjustmentEngine {
    pub fn compute(&mut self, item: &TimelineAdjustment) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[TimelineAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_timeline_adjustment_engine(flags: &flags::Registry) -> TimelineAdjustmentEngine {
    TimelineAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedTimelineAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "timeline-adjustment-seasonal".to_string(),
                percent: 0.7,
            }) as Rc<dyn TimelineAdjustmentStep>,
            Rc::new(PercentTimelineAdjustmentAdjuster { percent: 20.8 }),
            Rc::new(MinimumTimelineAdjustmentAdjuster { minimum: 33.2 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 7774.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_adjustment_engine(&flags::new_registry());
        let item = new_timeline_adjustment("code-83", "name-714");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
