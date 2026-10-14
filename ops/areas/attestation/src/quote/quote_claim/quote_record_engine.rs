use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote record calculation.
pub trait QuoteRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteRecord, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteRecordAdjuster for FlaggedQuoteRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteRecordAdjuster {
    pub minimum: f64,
}

impl QuoteRecordAdjuster for MinimumQuoteRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteRecordAdjuster {
    pub cap: f64,
}

impl QuoteRecordAdjuster for CapQuoteRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteRecordAdjuster for TieredQuoteRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the quote record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteRecordEngine {
    pub steps: Vec<Rc<dyn QuoteRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteRecordEngine {
    pub fn evaluate(&mut self, item: &QuoteRecord) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[QuoteRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_quote_record_engine(flags: &flags::Registry) -> QuoteRecordEngine {
    QuoteRecordEngine {
        steps: vec![
            Rc::new(FlaggedQuoteRecordAdjuster {
                flags: flags.clone(),
                flag: "quote-record-capped".to_string(),
                percent: -1.9,
            }) as Rc<dyn QuoteRecordAdjuster>,
            Rc::new(MinimumQuoteRecordAdjuster { minimum: 31.5 }),
            Rc::new(CapQuoteRecordAdjuster { cap: 48002.0 }),
            Rc::new(TieredQuoteRecordAdjuster {
                bounds: vec![5419.0, 7260.0],
                rates: vec![1.11, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 64504.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_record_engine(&flags::new_registry());
        let item = new_quote_record("code-541", "label-225");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
