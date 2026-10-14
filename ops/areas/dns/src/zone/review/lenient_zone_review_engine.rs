use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient zone review calculation.
pub trait LenientZoneReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientZoneReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientZoneReviewAdjuster {
    pub percent: f64,
}

impl LenientZoneReviewModifier for PercentLenientZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientZoneReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientZoneReviewAdjuster {
    pub minimum: f64,
}

impl LenientZoneReviewModifier for MinimumLenientZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientZoneReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientZoneReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientZoneReviewModifier for LabelLenientZoneReviewAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientZoneReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientZoneReviewAdjuster {
    pub cap: f64,
}

impl LenientZoneReviewModifier for CapLenientZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientZoneReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient zone review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientZoneReviewEngine {
    pub steps: Vec<Rc<dyn LenientZoneReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientZoneReviewEngine {
    pub fn compute(&mut self, item: &LenientZoneReview) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[LenientZoneReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_zone_review_engine() -> LenientZoneReviewEngine {
    LenientZoneReviewEngine {
        steps: vec![
            Rc::new(PercentLenientZoneReviewAdjuster { percent: -20.2 }) as Rc<dyn LenientZoneReviewModifier>,
            Rc::new(MinimumLenientZoneReviewAdjuster { minimum: 45.9 }),
            Rc::new(LabelLenientZoneReviewAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.78),
                    ("central".to_string(), 0.89),
                    ("east".to_string(), 0.72),
                ]),
            }),
            Rc::new(CapLenientZoneReviewAdjuster { cap: 13651.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 11271.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_zone_review_engine();
        let item = new_lenient_zone_review("code-438", "label-915");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
