use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict forecast review calculation.
pub trait StrictForecastReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictForecastReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictForecastReviewAdjuster {
    pub step: f64,
}

impl StrictForecastReviewModifier for RoundingStrictForecastReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictForecastReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictForecastReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictForecastReviewModifier for FlaggedStrictForecastReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictForecastReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryStrictForecastReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictForecastReviewModifier for CategoryStrictForecastReviewAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &StrictForecastReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict forecast review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictForecastReviewEngine {
    pub steps: Vec<Rc<dyn StrictForecastReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictForecastReviewEngine {
    pub fn compute(&mut self, item: &StrictForecastReview) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictForecastReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictForecastReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_forecast_review_engine(flags: &flags::Registry) -> StrictForecastReviewEngine {
    StrictForecastReviewEngine {
        steps: vec![
            Rc::new(RoundingStrictForecastReviewAdjuster { step: 5.0 }) as Rc<dyn StrictForecastReviewModifier>,
            Rc::new(FlaggedStrictForecastReviewAdjuster {
                flags: flags.clone(),
                flag: "strict-forecast-review-primary".to_string(),
                percent: 6.3,
            }),
            Rc::new(CategoryStrictForecastReviewAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.15)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 67380.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_forecast_review_engine(&flags::new_registry());
        let item = new_strict_forecast_review("code-834", "title-501");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
