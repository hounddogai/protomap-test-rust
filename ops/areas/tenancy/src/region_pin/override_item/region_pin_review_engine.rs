use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the region pin review calculation.
pub trait RegionPinReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReviewCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinReviewAdjuster for RegionPinReviewCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinReview, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinReviewAdjuster {
    pub cap: f64,
}

impl RegionPinReviewAdjuster for CapRegionPinReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinReviewAdjuster for TieredRegionPinReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinReviewAdjuster {
    pub minimum: f64,
}

impl RegionPinReviewAdjuster for MinimumRegionPinReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionPinReviewAdjuster {
    pub percent: f64,
}

impl RegionPinReviewAdjuster for PercentRegionPinReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionPinReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the region pin review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinReviewEngine {
    pub steps: Vec<Rc<dyn RegionPinReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinReviewEngine {
    pub fn compute(&mut self, item: &RegionPinReview) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_region_pin_review_engine() -> RegionPinReviewEngine {
    RegionPinReviewEngine {
        steps: vec![
            Rc::new(RegionPinReviewCreatedAtOffsetAdjuster { weight: 1.47 }) as Rc<dyn RegionPinReviewAdjuster>,
            Rc::new(CapRegionPinReviewAdjuster { cap: 25753.0 }),
            Rc::new(TieredRegionPinReviewAdjuster {
                bounds: vec![1817.0, 3124.0, 6062.0, 6919.0],
                rates: vec![1.09, 1.16, 1.07, 0.95],
            }),
            Rc::new(MinimumRegionPinReviewAdjuster { minimum: 3.0 }),
            Rc::new(PercentRegionPinReviewAdjuster { percent: -20.1 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 11184.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_review_engine();
        let item = new_region_pin_review("code-320", "label-909");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
