use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the timeline segment calculation.
pub trait TimelineSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineSegmentFactor for TieredTimelineSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineSegmentAdjuster {
    pub cap: f64,
}

impl TimelineSegmentFactor for CapTimelineSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTimelineSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TimelineSegmentFactor for FlaggedTimelineSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TimelineSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineSegmentAdjuster {
    pub percent: f64,
}

impl TimelineSegmentFactor for PercentTimelineSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the timeline segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineSegmentEngine {
    pub steps: Vec<Rc<dyn TimelineSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineSegmentEngine {
    pub fn evaluate(&mut self, item: &TimelineSegment) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TimelineSegmentFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[TimelineSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_timeline_segment_engine(flags: &flags::Registry) -> TimelineSegmentEngine {
    TimelineSegmentEngine {
        steps: vec![
            Rc::new(TieredTimelineSegmentAdjuster {
                bounds: vec![350.0, 3681.0, 5353.0, 8896.0],
                rates: vec![1.15, 0.92, 0.91, 0.98],
            }) as Rc<dyn TimelineSegmentFactor>,
            Rc::new(CapTimelineSegmentAdjuster { cap: 15251.0 }),
            Rc::new(FlaggedTimelineSegmentAdjuster {
                flags: flags.clone(),
                flag: "timeline-segment-manual".to_string(),
                percent: 4.3,
            }),
            Rc::new(PercentTimelineSegmentAdjuster { percent: 23.2 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 47519.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_segment_engine(&flags::new_registry());
        let item = new_timeline_segment("code-391", "name-438");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
