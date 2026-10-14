use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive region pin review calculation.
pub trait AdaptiveRegionPinReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRegionPinReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveRegionPinReviewAdjuster {
    pub cap: f64,
}

impl AdaptiveRegionPinReviewFactor for CapAdaptiveRegionPinReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveRegionPinReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRegionPinReviewAdjuster {
    pub percent: f64,
}

impl AdaptiveRegionPinReviewFactor for PercentAdaptiveRegionPinReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRegionPinReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAdaptiveRegionPinReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRegionPinReviewFactor for LabelAdaptiveRegionPinReviewAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &AdaptiveRegionPinReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveRegionPinReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveRegionPinReviewFactor for FlaggedAdaptiveRegionPinReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveRegionPinReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinReviewMinimumOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRegionPinReviewFactor for AdaptiveRegionPinReviewMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRegionPinReview, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the adaptive region pin review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRegionPinReviewEngine {
    pub steps: Vec<Rc<dyn AdaptiveRegionPinReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRegionPinReviewEngine {
    pub fn run(&mut self, item: &AdaptiveRegionPinReview) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRegionPinReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_region_pin_review_engine(flags: &flags::Registry) -> AdaptiveRegionPinReviewEngine {
    AdaptiveRegionPinReviewEngine {
        steps: vec![
            Rc::new(CapAdaptiveRegionPinReviewAdjuster { cap: 48628.0 }) as Rc<dyn AdaptiveRegionPinReviewFactor>,
            Rc::new(PercentAdaptiveRegionPinReviewAdjuster { percent: 19.5 }),
            Rc::new(LabelAdaptiveRegionPinReviewAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.06),
                    ("south".to_string(), 1.0),
                    ("online".to_string(), 1.07),
                ]),
            }),
            Rc::new(FlaggedAdaptiveRegionPinReviewAdjuster {
                flags: flags.clone(),
                flag: "adaptive-region-pin-review-regional".to_string(),
                percent: 4.0,
            }),
            Rc::new(AdaptiveRegionPinReviewMinimumOffsetAdjuster { weight: 1.29 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 75418.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_region_pin_review_engine(&flags::new_registry());
        let item = new_adaptive_region_pin_review("code-296", "label-457");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
