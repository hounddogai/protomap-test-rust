use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global plan review calculation.
pub trait GlobalPlanReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPlanReview, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalPlanReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalPlanReviewModifier for FlaggedGlobalPlanReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalPlanReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanReviewDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPlanReviewModifier for GlobalPlanReviewDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPlanReview, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalPlanReviewAdjuster {
    pub step: f64,
}

impl GlobalPlanReviewModifier for RoundingGlobalPlanReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalPlanReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global plan review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPlanReviewEngine {
    pub steps: Vec<Rc<dyn GlobalPlanReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPlanReviewEngine {
    pub fn run(&mut self, item: &GlobalPlanReview) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalPlanReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPlanReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_plan_review_engine(flags: &flags::Registry) -> GlobalPlanReviewEngine {
    GlobalPlanReviewEngine {
        steps: vec![
            Rc::new(FlaggedGlobalPlanReviewAdjuster {
                flags: flags.clone(),
                flag: "global-plan-review-global".to_string(),
                percent: -9.9,
            }) as Rc<dyn GlobalPlanReviewModifier>,
            Rc::new(GlobalPlanReviewDurationMinutesOffsetAdjuster { weight: 1.43 }),
            Rc::new(RoundingGlobalPlanReviewAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 45595.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_plan_review_engine(&flags::new_registry());
        let item = new_global_plan_review("code-67", "name-423");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
