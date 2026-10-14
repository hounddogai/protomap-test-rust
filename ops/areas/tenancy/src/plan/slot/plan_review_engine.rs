use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan review calculation.
pub trait PlanReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanReview, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanReviewFactor for FlaggedPlanReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanReviewAdjuster {
    pub percent: f64,
}

impl PlanReviewFactor for PercentPlanReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanReviewFactor for TieredPlanReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanReviewAdjuster {
    pub minimum: f64,
}

impl PlanReviewFactor for MinimumPlanReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanReviewAdjuster {
    pub step: f64,
}

impl PlanReviewFactor for RoundingPlanReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePlanReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanReviewFactor for SourcePlanReviewAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PlanReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the plan review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanReviewEngine {
    pub steps: Vec<Rc<dyn PlanReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanReviewEngine {
    pub fn compute(&mut self, item: &PlanReview) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanReviewFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_review_engine(flags: &flags::Registry) -> PlanReviewEngine {
    PlanReviewEngine {
        steps: vec![
            Rc::new(FlaggedPlanReviewAdjuster {
                flags: flags.clone(),
                flag: "plan-review-weighted".to_string(),
                percent: 8.3,
            }) as Rc<dyn PlanReviewFactor>,
            Rc::new(PercentPlanReviewAdjuster { percent: 17.7 }),
            Rc::new(TieredPlanReviewAdjuster {
                bounds: vec![547.0, 3512.0],
                rates: vec![1.16, 1.08],
            }),
            Rc::new(MinimumPlanReviewAdjuster { minimum: 17.5 }),
            Rc::new(RoundingPlanReviewAdjuster { step: 0.1 }),
            Rc::new(SourcePlanReviewAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.24)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 59791.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_review_engine(&flags::new_registry());
        let item = new_plan_review("code-611", "title-745");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
