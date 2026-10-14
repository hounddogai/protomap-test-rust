use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline request calculation.
pub trait TimelineRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRequestEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl TimelineRequestStep for TimelineRequestEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineRequest, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineRequestAdjuster {
    pub percent: f64,
}

impl TimelineRequestStep for PercentTimelineRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineRequestAdjuster {
    pub step: f64,
}

impl TimelineRequestStep for RoundingTimelineRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the timeline request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineRequestEngine {
    pub steps: Vec<Rc<dyn TimelineRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineRequestEngine {
    pub fn run(&mut self, item: &TimelineRequest) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[TimelineRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_timeline_request_engine() -> TimelineRequestEngine {
    TimelineRequestEngine {
        steps: vec![
            Rc::new(TimelineRequestEndsAtOffsetAdjuster { weight: 0.57 }) as Rc<dyn TimelineRequestStep>,
            Rc::new(PercentTimelineRequestAdjuster { percent: -17.2 }),
            Rc::new(RoundingTimelineRequestAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 2075.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_request_engine();
        let item = new_timeline_request("code-211", "title-527");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
