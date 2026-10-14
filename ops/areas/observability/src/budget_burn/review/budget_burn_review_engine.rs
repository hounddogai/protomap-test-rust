use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn review calculation.
pub trait BudgetBurnReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReviewMaximumOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnReviewStep for BudgetBurnReviewMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnReview, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnReviewAdjuster {
    pub minimum: f64,
}

impl BudgetBurnReviewStep for MinimumBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnReviewStep for FlaggedBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnReviewAdjuster {
    pub step: f64,
}

impl BudgetBurnReviewStep for RoundingBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnReviewAdjuster {
    pub percent: f64,
}

impl BudgetBurnReviewStep for PercentBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleBudgetBurnReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnReviewStep for LocaleBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &BudgetBurnReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the budget burn review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnReviewEngine {
    pub steps: Vec<Rc<dyn BudgetBurnReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnReviewEngine {
    pub fn price(&mut self, item: &BudgetBurnReview) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnReviewStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_budget_burn_review_engine(flags: &flags::Registry) -> BudgetBurnReviewEngine {
    BudgetBurnReviewEngine {
        steps: vec![
            Rc::new(BudgetBurnReviewMaximumOffsetAdjuster { weight: 0.02 }) as Rc<dyn BudgetBurnReviewStep>,
            Rc::new(MinimumBudgetBurnReviewAdjuster { minimum: 4.7 }),
            Rc::new(FlaggedBudgetBurnReviewAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-review-regional".to_string(),
                percent: 9.1,
            }),
            Rc::new(RoundingBudgetBurnReviewAdjuster { step: 0.1 }),
            Rc::new(PercentBudgetBurnReviewAdjuster { percent: 19.2 }),
            Rc::new(LocaleBudgetBurnReviewAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.05)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 2549.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_review_engine(&flags::new_registry());
        let item = new_budget_burn_review("code-454", "title-984");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
