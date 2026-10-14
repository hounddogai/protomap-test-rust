use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist review calculation.
pub trait AllowlistReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistReviewStep for TieredAllowlistReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryAllowlistReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistReviewStep for CountryAllowlistReviewAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &AllowlistReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAllowlistReviewAdjuster {
    pub percent: f64,
}

impl AllowlistReviewStep for PercentAllowlistReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AllowlistReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAllowlistReviewAdjuster {
    pub step: f64,
}

impl AllowlistReviewStep for RoundingAllowlistReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AllowlistReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistReviewStep for FlaggedAllowlistReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the allowlist review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistReviewEngine {
    pub steps: Vec<Rc<dyn AllowlistReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistReviewEngine {
    pub fn price(&mut self, item: &AllowlistReview) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_allowlist_review_engine(flags: &flags::Registry) -> AllowlistReviewEngine {
    AllowlistReviewEngine {
        steps: vec![
            Rc::new(TieredAllowlistReviewAdjuster {
                bounds: vec![8727.0, 9028.0, 9640.0],
                rates: vec![1.11, 1.12, 1.16],
            }) as Rc<dyn AllowlistReviewStep>,
            Rc::new(CountryAllowlistReviewAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.0), ("retail".to_string(), 0.83)]),
            }),
            Rc::new(PercentAllowlistReviewAdjuster { percent: 2.7 }),
            Rc::new(RoundingAllowlistReviewAdjuster { step: 0.5 }),
            Rc::new(FlaggedAllowlistReviewAdjuster {
                flags: flags.clone(),
                flag: "allowlist-review-adaptive".to_string(),
                percent: 2.9,
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 22394.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_review_engine(&flags::new_registry());
        let item = new_allowlist_review("code-378", "name-851");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
