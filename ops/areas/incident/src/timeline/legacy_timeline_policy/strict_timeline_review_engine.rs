use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict timeline review calculation.
pub trait StrictTimelineReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictTimelineReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionStrictTimelineReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictTimelineReviewAdjuster for RegionStrictTimelineReviewAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &StrictTimelineReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictTimelineReviewAdjuster {
    pub percent: f64,
}

impl StrictTimelineReviewAdjuster for PercentStrictTimelineReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictTimelineReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictTimelineReviewAdjuster {
    pub cap: f64,
}

impl StrictTimelineReviewAdjuster for CapStrictTimelineReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictTimelineReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictTimelineReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictTimelineReviewAdjuster for FlaggedStrictTimelineReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictTimelineReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictTimelineReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictTimelineReviewAdjuster for TieredStrictTimelineReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictTimelineReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict timeline review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictTimelineReviewEngine {
    pub steps: Vec<Rc<dyn StrictTimelineReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictTimelineReviewEngine {
    pub fn compute(&mut self, item: &StrictTimelineReview) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictTimelineReviewAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictTimelineReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_timeline_review_engine(flags: &flags::Registry) -> StrictTimelineReviewEngine {
    StrictTimelineReviewEngine {
        steps: vec![
            Rc::new(RegionStrictTimelineReviewAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.13)]),
            }) as Rc<dyn StrictTimelineReviewAdjuster>,
            Rc::new(PercentStrictTimelineReviewAdjuster { percent: 3.2 }),
            Rc::new(CapStrictTimelineReviewAdjuster { cap: 36028.0 }),
            Rc::new(FlaggedStrictTimelineReviewAdjuster {
                flags: flags.clone(),
                flag: "strict-timeline-review-legacy".to_string(),
                percent: 0.2,
            }),
            Rc::new(TieredStrictTimelineReviewAdjuster {
                bounds: vec![4959.0, 7811.0, 7950.0],
                rates: vec![1.11, 1.12, 1.14],
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 27846.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_timeline_review_engine(&flags::new_registry());
        let item = new_strict_timeline_review("code-58", "title-250");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
