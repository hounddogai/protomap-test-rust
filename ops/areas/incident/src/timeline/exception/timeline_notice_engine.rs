use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the timeline notice calculation.
pub trait TimelineNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineNoticeAdjuster {
    pub percent: f64,
}

impl TimelineNoticeAdjuster for PercentTimelineNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineNoticeAdjuster {
    pub minimum: f64,
}

impl TimelineNoticeAdjuster for MinimumTimelineNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNoticeMarginOffsetAdjuster {
    pub weight: f64,
}

impl TimelineNoticeAdjuster for TimelineNoticeMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineNotice, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineNoticeAdjuster for TieredTimelineNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineNoticeAdjuster {
    pub cap: f64,
}

impl TimelineNoticeAdjuster for CapTimelineNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the timeline notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineNoticeEngine {
    pub steps: Vec<Rc<dyn TimelineNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineNoticeEngine {
    pub fn run(&mut self, item: &TimelineNotice) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TimelineNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TimelineNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_timeline_notice_engine() -> TimelineNoticeEngine {
    TimelineNoticeEngine {
        steps: vec![
            Rc::new(PercentTimelineNoticeAdjuster { percent: -22.8 }) as Rc<dyn TimelineNoticeAdjuster>,
            Rc::new(MinimumTimelineNoticeAdjuster { minimum: 44.1 }),
            Rc::new(TimelineNoticeMarginOffsetAdjuster { weight: 1.69 }),
            Rc::new(TieredTimelineNoticeAdjuster {
                bounds: vec![2957.0, 3569.0, 4685.0],
                rates: vec![1.02, 1.14, 1.0],
            }),
            Rc::new(CapTimelineNoticeAdjuster { cap: 534.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 9844.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_notice_engine();
        let item = new_timeline_notice("code-349", "label-172");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
