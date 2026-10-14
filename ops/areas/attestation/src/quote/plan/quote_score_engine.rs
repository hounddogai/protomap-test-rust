use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote score calculation.
pub trait QuoteScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScoreDiscountOffsetAdjuster {
    pub weight: f64,
}

impl QuoteScoreStep for QuoteScoreDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &QuoteScore, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteScoreAdjuster {
    pub minimum: f64,
}

impl QuoteScoreStep for MinimumQuoteScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteScoreAdjuster {
    pub percent: f64,
}

impl QuoteScoreStep for PercentQuoteScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteScoreAdjuster {
    pub step: f64,
}

impl QuoteScoreStep for RoundingQuoteScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryQuoteScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteScoreStep for CountryQuoteScoreAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &QuoteScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteScoreStep for FlaggedQuoteScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the quote score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteScoreEngine {
    pub steps: Vec<Rc<dyn QuoteScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteScoreEngine {
    pub fn run(&mut self, item: &QuoteScore) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteScoreStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_score_engine(flags: &flags::Registry) -> QuoteScoreEngine {
    QuoteScoreEngine {
        steps: vec![
            Rc::new(QuoteScoreDiscountOffsetAdjuster { weight: 1.61 }) as Rc<dyn QuoteScoreStep>,
            Rc::new(MinimumQuoteScoreAdjuster { minimum: 42.3 }),
            Rc::new(PercentQuoteScoreAdjuster { percent: -14.5 }),
            Rc::new(RoundingQuoteScoreAdjuster { step: 1.0 }),
            Rc::new(CountryQuoteScoreAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.83), ("west".to_string(), 0.84)]),
            }),
            Rc::new(FlaggedQuoteScoreAdjuster {
                flags: flags.clone(),
                flag: "quote-score-legacy".to_string(),
                percent: 6.9,
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 75541.4,
    }
}
