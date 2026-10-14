use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary segment calculation.
pub trait CanarySegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanarySegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySegmentRevisionOffsetAdjuster {
    pub weight: f64,
}

impl CanarySegmentAdjuster for CanarySegmentRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &CanarySegment, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanarySegmentAdjuster {
    pub step: f64,
}

impl CanarySegmentAdjuster for RoundingCanarySegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanarySegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanarySegmentAdjuster {
    pub minimum: f64,
}

impl CanarySegmentAdjuster for MinimumCanarySegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanarySegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanarySegmentAdjuster {
    pub percent: f64,
}

impl CanarySegmentAdjuster for PercentCanarySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanarySegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanarySegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanarySegmentAdjuster for FlaggedCanarySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanarySegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the canary segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanarySegmentEngine {
    pub steps: Vec<Rc<dyn CanarySegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanarySegmentEngine {
    pub fn compute(&mut self, item: &CanarySegment) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CanarySegmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CanarySegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_canary_segment_engine(flags: &flags::Registry) -> CanarySegmentEngine {
    CanarySegmentEngine {
        steps: vec![
            Rc::new(CanarySegmentRevisionOffsetAdjuster { weight: 0.77 }) as Rc<dyn CanarySegmentAdjuster>,
            Rc::new(RoundingCanarySegmentAdjuster { step: 0.05 }),
            Rc::new(MinimumCanarySegmentAdjuster { minimum: 37.6 }),
            Rc::new(PercentCanarySegmentAdjuster { percent: 20.8 }),
            Rc::new(FlaggedCanarySegmentAdjuster {
                flags: flags.clone(),
                flag: "canary-segment-primary".to_string(),
                percent: -9.4,
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 93096.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_segment_engine(&flags::new_registry());
        let item = new_canary_segment("code-709", "title-612");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
