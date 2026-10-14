use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled sink review calculation.
pub trait ScheduledSinkReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSinkReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScheduledSinkReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledSinkReviewFactor for TitleScheduledSinkReviewAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ScheduledSinkReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledSinkReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledSinkReviewFactor for FlaggedScheduledSinkReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledSinkReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledSinkReviewAdjuster {
    pub percent: f64,
}

impl ScheduledSinkReviewFactor for PercentScheduledSinkReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledSinkReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledSinkReviewAdjuster {
    pub minimum: f64,
}

impl ScheduledSinkReviewFactor for MinimumScheduledSinkReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledSinkReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkReviewRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledSinkReviewFactor for ScheduledSinkReviewRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledSinkReview, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledSinkReviewAdjuster {
    pub cap: f64,
}

impl ScheduledSinkReviewFactor for CapScheduledSinkReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledSinkReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled sink review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSinkReviewEngine {
    pub steps: Vec<Rc<dyn ScheduledSinkReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSinkReviewEngine {
    pub fn price(&mut self, item: &ScheduledSinkReview) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledSinkReviewFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSinkReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_sink_review_engine(flags: &flags::Registry) -> ScheduledSinkReviewEngine {
    ScheduledSinkReviewEngine {
        steps: vec![
            Rc::new(TitleScheduledSinkReviewAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.83)]),
            }) as Rc<dyn ScheduledSinkReviewFactor>,
            Rc::new(FlaggedScheduledSinkReviewAdjuster {
                flags: flags.clone(),
                flag: "scheduled-sink-review-pending".to_string(),
                percent: 2.8,
            }),
            Rc::new(PercentScheduledSinkReviewAdjuster { percent: -1.8 }),
            Rc::new(MinimumScheduledSinkReviewAdjuster { minimum: 47.0 }),
            Rc::new(ScheduledSinkReviewRotationDaysOffsetAdjuster { weight: 0.08 }),
            Rc::new(CapScheduledSinkReviewAdjuster { cap: 14046.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 69954.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_sink_review_engine(&flags::new_registry());
        let item = new_scheduled_sink_review("code-600", "title-883");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
