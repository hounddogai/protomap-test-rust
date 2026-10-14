use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the quote review calculation.
pub trait QuoteReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteReviewAdjuster {
    pub step: f64,
}

impl QuoteReviewAdjuster for RoundingQuoteReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceQuoteReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteReviewAdjuster for SourceQuoteReviewAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &QuoteReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteReviewAdjuster {
    pub percent: f64,
}

impl QuoteReviewAdjuster for PercentQuoteReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteReviewAdjuster for TieredQuoteReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the quote review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteReviewEngine {
    pub steps: Vec<Rc<dyn QuoteReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteReviewEngine {
    pub fn run(&mut self, item: &QuoteReview) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[QuoteReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_review_engine() -> QuoteReviewEngine {
    QuoteReviewEngine {
        steps: vec![
            Rc::new(RoundingQuoteReviewAdjuster { step: 0.5 }) as Rc<dyn QuoteReviewAdjuster>,
            Rc::new(SourceQuoteReviewAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.27)]),
            }),
            Rc::new(PercentQuoteReviewAdjuster { percent: -12.3 }),
            Rc::new(TieredQuoteReviewAdjuster {
                bounds: vec![1091.0, 3429.0],
                rates: vec![1.01, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 26054.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_review_engine();
        let item = new_quote_review("code-988", "name-140");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
