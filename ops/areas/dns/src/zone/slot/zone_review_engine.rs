use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the zone review calculation.
pub trait ZoneReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneReview, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneReviewModifier for FlaggedZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneReviewAdjuster {
    pub step: f64,
}

impl ZoneReviewModifier for RoundingZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneReviewAdjuster {
    pub percent: f64,
}

impl ZoneReviewModifier for PercentZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReviewCostOffsetAdjuster {
    pub weight: f64,
}

impl ZoneReviewModifier for ZoneReviewCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneReview, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneReviewAdjuster {
    pub cap: f64,
}

impl ZoneReviewModifier for CapZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneReviewAdjuster {
    pub minimum: f64,
}

impl ZoneReviewModifier for MinimumZoneReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the zone review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneReviewEngine {
    pub steps: Vec<Rc<dyn ZoneReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneReviewEngine {
    pub fn evaluate(&mut self, item: &ZoneReview) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_zone_review_engine(flags: &flags::Registry) -> ZoneReviewEngine {
    ZoneReviewEngine {
        steps: vec![
            Rc::new(FlaggedZoneReviewAdjuster {
                flags: flags.clone(),
                flag: "zone-review-manual".to_string(),
                percent: 9.8,
            }) as Rc<dyn ZoneReviewModifier>,
            Rc::new(RoundingZoneReviewAdjuster { step: 0.1 }),
            Rc::new(PercentZoneReviewAdjuster { percent: 12.8 }),
            Rc::new(ZoneReviewCostOffsetAdjuster { weight: 0.41 }),
            Rc::new(CapZoneReviewAdjuster { cap: 11406.0 }),
            Rc::new(MinimumZoneReviewAdjuster { minimum: 34.0 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 89138.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_review_engine(&flags::new_registry());
        let item = new_zone_review("code-632", "name-483");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
