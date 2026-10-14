use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the timeline exception calculation.
pub trait TimelineExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTimelineExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TimelineExceptionAdjuster for ScopeTimelineExceptionAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TimelineException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineExceptionAdjuster {
    pub minimum: f64,
}

impl TimelineExceptionAdjuster for MinimumTimelineExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTimelineExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TimelineExceptionAdjuster for FlaggedTimelineExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TimelineException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineExceptionAdjuster {
    pub step: f64,
}

impl TimelineExceptionAdjuster for RoundingTimelineExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineExceptionCapacityOffsetAdjuster {
    pub weight: f64,
}

impl TimelineExceptionAdjuster for TimelineExceptionCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineException, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineExceptionAdjuster {
    pub percent: f64,
}

impl TimelineExceptionAdjuster for PercentTimelineExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the timeline exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineExceptionEngine {
    pub steps: Vec<Rc<dyn TimelineExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineExceptionEngine {
    pub fn evaluate(&mut self, item: &TimelineException) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TimelineException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_timeline_exception_engine(flags: &flags::Registry) -> TimelineExceptionEngine {
    TimelineExceptionEngine {
        steps: vec![
            Rc::new(ScopeTimelineExceptionAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.08), ("central".to_string(), 0.88)]),
            }) as Rc<dyn TimelineExceptionAdjuster>,
            Rc::new(MinimumTimelineExceptionAdjuster { minimum: 19.9 }),
            Rc::new(FlaggedTimelineExceptionAdjuster {
                flags: flags.clone(),
                flag: "timeline-exception-primary".to_string(),
                percent: -5.9,
            }),
            Rc::new(RoundingTimelineExceptionAdjuster { step: 5.0 }),
            Rc::new(TimelineExceptionCapacityOffsetAdjuster { weight: 0.56 }),
            Rc::new(PercentTimelineExceptionAdjuster { percent: 19.8 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 49370.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_exception_engine(&flags::new_registry());
        let item = new_timeline_exception("code-485", "title-44");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
