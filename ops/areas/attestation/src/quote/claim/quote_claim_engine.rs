use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote claim calculation.
pub trait QuoteClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteClaimAdjuster for TieredQuoteClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteClaimAdjuster for FlaggedQuoteClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteClaimAdjuster {
    pub minimum: f64,
}

impl QuoteClaimAdjuster for MinimumQuoteClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteClaimAdjuster {
    pub step: f64,
}

impl QuoteClaimAdjuster for RoundingQuoteClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteClaimAdjuster {
    pub percent: f64,
}

impl QuoteClaimAdjuster for PercentQuoteClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the quote claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteClaimEngine {
    pub steps: Vec<Rc<dyn QuoteClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteClaimEngine {
    pub fn price(&mut self, item: &QuoteClaim) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[QuoteClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_quote_claim_engine(flags: &flags::Registry) -> QuoteClaimEngine {
    QuoteClaimEngine {
        steps: vec![
            Rc::new(TieredQuoteClaimAdjuster {
                bounds: vec![4140.0, 7271.0, 7598.0, 9913.0],
                rates: vec![1.17, 1.09, 1.15, 1.16],
            }) as Rc<dyn QuoteClaimAdjuster>,
            Rc::new(FlaggedQuoteClaimAdjuster {
                flags: flags.clone(),
                flag: "quote-claim-adaptive".to_string(),
                percent: 6.5,
            }),
            Rc::new(MinimumQuoteClaimAdjuster { minimum: 3.5 }),
            Rc::new(RoundingQuoteClaimAdjuster { step: 0.05 }),
            Rc::new(PercentQuoteClaimAdjuster { percent: 1.6 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 98823.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_claim_engine(&flags::new_registry());
        let item = new_quote_claim("code-780", "label-629");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
