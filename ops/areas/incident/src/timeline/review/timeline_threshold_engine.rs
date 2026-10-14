use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the timeline threshold calculation.
pub trait TimelineThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTimelineThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TimelineThresholdAdjuster for FlaggedTimelineThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TimelineThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineThresholdAdjuster {
    pub cap: f64,
}

impl TimelineThresholdAdjuster for CapTimelineThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineThresholdAdjuster {
    pub percent: f64,
}

impl TimelineThresholdAdjuster for PercentTimelineThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the timeline threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineThresholdEngine {
    pub steps: Vec<Rc<dyn TimelineThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineThresholdEngine {
    pub fn run(&mut self, item: &TimelineThreshold) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[TimelineThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_timeline_threshold_engine(flags: &flags::Registry) -> TimelineThresholdEngine {
    TimelineThresholdEngine {
        steps: vec![
            Rc::new(FlaggedTimelineThresholdAdjuster {
                flags: flags.clone(),
                flag: "timeline-threshold-scheduled".to_string(),
                percent: -5.1,
            }) as Rc<dyn TimelineThresholdAdjuster>,
            Rc::new(CapTimelineThresholdAdjuster { cap: 11921.0 }),
            Rc::new(PercentTimelineThresholdAdjuster { percent: -3.6 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 39363.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_threshold_engine(&flags::new_registry());
        let item = new_timeline_threshold("code-761", "title-78");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
