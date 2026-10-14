use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the quote entry calculation.
pub trait QuoteEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelQuoteEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteEntryAdjuster for ChannelQuoteEntryAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &QuoteEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteEntryAdjuster for TieredQuoteEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteEntryAdjuster {
    pub percent: f64,
}

impl QuoteEntryAdjuster for PercentQuoteEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteEntryAdjuster {
    pub cap: f64,
}

impl QuoteEntryAdjuster for CapQuoteEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteEntryAdjuster {
    pub minimum: f64,
}

impl QuoteEntryAdjuster for MinimumQuoteEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the quote entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteEntryEngine {
    pub steps: Vec<Rc<dyn QuoteEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteEntryEngine {
    pub fn run(&mut self, item: &QuoteEntry) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_entry_engine() -> QuoteEntryEngine {
    QuoteEntryEngine {
        steps: vec![
            Rc::new(ChannelQuoteEntryAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.94)]),
            }) as Rc<dyn QuoteEntryAdjuster>,
            Rc::new(TieredQuoteEntryAdjuster {
                bounds: vec![5295.0, 8225.0, 8293.0],
                rates: vec![1.12, 0.97, 0.94],
            }),
            Rc::new(PercentQuoteEntryAdjuster { percent: -13.1 }),
            Rc::new(CapQuoteEntryAdjuster { cap: 41091.0 }),
            Rc::new(MinimumQuoteEntryAdjuster { minimum: 10.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 45580.2,
    }
}
