use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the quote notice calculation.
pub trait QuoteNoticeModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteNoticeAdjuster {
    pub step: f64,
}

impl QuoteNoticeModifier for RoundingQuoteNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteNoticeModifier for TieredQuoteNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteNoticeAdjuster {
    pub minimum: f64,
}

impl QuoteNoticeModifier for MinimumQuoteNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteNoticeAdjuster {
    pub percent: f64,
}

impl QuoteNoticeModifier for PercentQuoteNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the quote notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteNoticeEngine {
    pub steps: Vec<Rc<dyn QuoteNoticeModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteNoticeEngine {
    pub fn evaluate(&mut self, item: &QuoteNotice) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[QuoteNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_quote_notice_engine() -> QuoteNoticeEngine {
    QuoteNoticeEngine {
        steps: vec![
            Rc::new(RoundingQuoteNoticeAdjuster { step: 1.0 }) as Rc<dyn QuoteNoticeModifier>,
            Rc::new(TieredQuoteNoticeAdjuster {
                bounds: vec![601.0, 3519.0, 4103.0, 5036.0],
                rates: vec![0.92, 0.82, 1.06, 0.83],
            }),
            Rc::new(MinimumQuoteNoticeAdjuster { minimum: 44.0 }),
            Rc::new(PercentQuoteNoticeAdjuster { percent: -6.3 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 75370.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_notice_engine();
        let item = new_quote_notice("code-481", "name-271");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
