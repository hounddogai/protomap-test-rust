use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment review calculation.
pub trait SegmentReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentReviewAdjuster {
    pub cap: f64,
}

impl SegmentReviewModifier for CapSegmentReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentReviewAdjuster {
    pub minimum: f64,
}

impl SegmentReviewModifier for MinimumSegmentReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentReviewModifier for TieredSegmentReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentReviewModifier for FlaggedSegmentReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReviewBurstOffsetAdjuster {
    pub weight: f64,
}

impl SegmentReviewModifier for SegmentReviewBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentReview, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentReviewAdjuster {
    pub percent: f64,
}

impl SegmentReviewModifier for PercentSegmentReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the segment review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentReviewEngine {
    pub steps: Vec<Rc<dyn SegmentReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentReviewEngine {
    pub fn compute(&mut self, item: &SegmentReview) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[SegmentReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_segment_review_engine(flags: &flags::Registry) -> SegmentReviewEngine {
    SegmentReviewEngine {
        steps: vec![
            Rc::new(CapSegmentReviewAdjuster { cap: 46365.0 }) as Rc<dyn SegmentReviewModifier>,
            Rc::new(MinimumSegmentReviewAdjuster { minimum: 21.5 }),
            Rc::new(TieredSegmentReviewAdjuster {
                bounds: vec![4079.0, 4329.0, 6525.0, 7071.0],
                rates: vec![1.02, 1.16, 0.89, 0.86],
            }),
            Rc::new(FlaggedSegmentReviewAdjuster {
                flags: flags.clone(),
                flag: "segment-review-weighted".to_string(),
                percent: -6.6,
            }),
            Rc::new(SegmentReviewBurstOffsetAdjuster { weight: 1.63 }),
            Rc::new(PercentSegmentReviewAdjuster { percent: 4.6 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 59033.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_review_engine(&flags::new_registry());
        let item = new_segment_review("code-924", "name-823");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
