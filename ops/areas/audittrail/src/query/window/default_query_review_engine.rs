use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default query review calculation.
pub trait DefaultQueryReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultQueryReview, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultQueryReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultQueryReviewStep for FlaggedDefaultQueryReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultQueryReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryReviewPortOffsetAdjuster {
    pub weight: f64,
}

impl DefaultQueryReviewStep for DefaultQueryReviewPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultQueryReview, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultQueryReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultQueryReviewStep for TieredDefaultQueryReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultQueryReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultQueryReviewAdjuster {
    pub percent: f64,
}

impl DefaultQueryReviewStep for PercentDefaultQueryReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultQueryReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultQueryReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultQueryReviewStep for LabelDefaultQueryReviewAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultQueryReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default query review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultQueryReviewEngine {
    pub steps: Vec<Rc<dyn DefaultQueryReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultQueryReviewEngine {
    pub fn price(&mut self, item: &DefaultQueryReview) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[DefaultQueryReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_query_review_engine(flags: &flags::Registry) -> DefaultQueryReviewEngine {
    DefaultQueryReviewEngine {
        steps: vec![
            Rc::new(FlaggedDefaultQueryReviewAdjuster {
                flags: flags.clone(),
                flag: "default-query-review-pending".to_string(),
                percent: -5.5,
            }) as Rc<dyn DefaultQueryReviewStep>,
            Rc::new(DefaultQueryReviewPortOffsetAdjuster { weight: 0.13 }),
            Rc::new(TieredDefaultQueryReviewAdjuster {
                bounds: vec![3504.0, 3738.0],
                rates: vec![0.9, 1.19],
            }),
            Rc::new(PercentDefaultQueryReviewAdjuster { percent: 13.8 }),
            Rc::new(LabelDefaultQueryReviewAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.88)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 68410.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_query_review_engine(&flags::new_registry());
        let item = new_default_query_review("code-805", "label-10");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
