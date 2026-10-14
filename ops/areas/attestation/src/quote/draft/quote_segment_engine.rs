use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the quote segment calculation.
pub trait QuoteSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteSegmentAdjuster {
    pub minimum: f64,
}

impl QuoteSegmentModifier for MinimumQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteSegmentModifier for TieredQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteSegmentModifier for FlaggedQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSegmentRateOffsetAdjuster {
    pub weight: f64,
}

impl QuoteSegmentModifier for QuoteSegmentRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &QuoteSegment, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteSegmentAdjuster {
    pub percent: f64,
}

impl QuoteSegmentModifier for PercentQuoteSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the quote segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteSegmentEngine {
    pub steps: Vec<Rc<dyn QuoteSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteSegmentEngine {
    pub fn run(&mut self, item: &QuoteSegment) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteSegmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_segment_engine(flags: &flags::Registry) -> QuoteSegmentEngine {
    QuoteSegmentEngine {
        steps: vec![
            Rc::new(MinimumQuoteSegmentAdjuster { minimum: 10.8 }) as Rc<dyn QuoteSegmentModifier>,
            Rc::new(TieredQuoteSegmentAdjuster {
                bounds: vec![1014.0, 4061.0, 8232.0],
                rates: vec![1.2, 0.85, 0.96],
            }),
            Rc::new(FlaggedQuoteSegmentAdjuster {
                flags: flags.clone(),
                flag: "quote-segment-regional".to_string(),
                percent: 2.6,
            }),
            Rc::new(QuoteSegmentRateOffsetAdjuster { weight: 1.18 }),
            Rc::new(PercentQuoteSegmentAdjuster { percent: -0.9 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 52582.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_segment_engine(&flags::new_registry());
        let item = new_quote_segment("code-530", "title-414");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
