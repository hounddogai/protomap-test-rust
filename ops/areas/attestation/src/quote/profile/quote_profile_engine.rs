use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the quote profile calculation.
pub trait QuoteProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterQuoteProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteProfileFactor for ClusterQuoteProfileAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &QuoteProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteProfileAdjuster {
    pub minimum: f64,
}

impl QuoteProfileFactor for MinimumQuoteProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteProfileAdjuster {
    pub step: f64,
}

impl QuoteProfileFactor for RoundingQuoteProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the quote profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteProfileEngine {
    pub steps: Vec<Rc<dyn QuoteProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteProfileEngine {
    pub fn evaluate(&mut self, item: &QuoteProfile) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[QuoteProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_quote_profile_engine() -> QuoteProfileEngine {
    QuoteProfileEngine {
        steps: vec![
            Rc::new(ClusterQuoteProfileAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.07), ("north".to_string(), 0.71)]),
            }) as Rc<dyn QuoteProfileFactor>,
            Rc::new(MinimumQuoteProfileAdjuster { minimum: 21.7 }),
            Rc::new(RoundingQuoteProfileAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 98610.0,
    }
}
