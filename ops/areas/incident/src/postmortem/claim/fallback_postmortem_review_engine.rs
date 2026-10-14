use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the fallback postmortem review calculation.
pub trait FallbackPostmortemReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPostmortemReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPostmortemReviewAdjuster {
    pub minimum: f64,
}

impl FallbackPostmortemReviewAdjuster for MinimumFallbackPostmortemReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPostmortemReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleFallbackPostmortemReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackPostmortemReviewAdjuster for LocaleFallbackPostmortemReviewAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &FallbackPostmortemReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemReviewDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl FallbackPostmortemReviewAdjuster for FallbackPostmortemReviewDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackPostmortemReview, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackPostmortemReviewAdjuster {
    pub percent: f64,
}

impl FallbackPostmortemReviewAdjuster for PercentFallbackPostmortemReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackPostmortemReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback postmortem review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPostmortemReviewEngine {
    pub steps: Vec<Rc<dyn FallbackPostmortemReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPostmortemReviewEngine {
    pub fn evaluate(&mut self, item: &FallbackPostmortemReview) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackPostmortemReviewAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPostmortemReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_postmortem_review_engine() -> FallbackPostmortemReviewEngine {
    FallbackPostmortemReviewEngine {
        steps: vec![
            Rc::new(MinimumFallbackPostmortemReviewAdjuster { minimum: 42.8 })
                as Rc<dyn FallbackPostmortemReviewAdjuster>,
            Rc::new(LocaleFallbackPostmortemReviewAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.81), ("west".to_string(), 0.78)]),
            }),
            Rc::new(FallbackPostmortemReviewDurationMinutesOffsetAdjuster { weight: 0.01 }),
            Rc::new(PercentFallbackPostmortemReviewAdjuster { percent: 17.5 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 88189.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_postmortem_review_engine();
        let item = new_fallback_postmortem_review("code-656", "title-492");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
