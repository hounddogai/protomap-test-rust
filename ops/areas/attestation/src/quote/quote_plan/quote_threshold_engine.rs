use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote threshold calculation.
pub trait QuoteThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteThresholdModifier for TieredQuoteThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteThresholdAdjuster {
    pub minimum: f64,
}

impl QuoteThresholdModifier for MinimumQuoteThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteThresholdAdjuster {
    pub step: f64,
}

impl QuoteThresholdModifier for RoundingQuoteThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteThresholdModifier for FlaggedQuoteThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleQuoteThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteThresholdModifier for LocaleQuoteThresholdAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &QuoteThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the quote threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteThresholdEngine {
    pub steps: Vec<Rc<dyn QuoteThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteThresholdEngine {
    pub fn price(&mut self, item: &QuoteThreshold) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteThresholdModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_quote_threshold_engine(flags: &flags::Registry) -> QuoteThresholdEngine {
    QuoteThresholdEngine {
        steps: vec![
            Rc::new(TieredQuoteThresholdAdjuster {
                bounds: vec![2534.0, 3206.0, 4132.0, 9911.0],
                rates: vec![0.81, 0.92, 1.13, 1.19],
            }) as Rc<dyn QuoteThresholdModifier>,
            Rc::new(MinimumQuoteThresholdAdjuster { minimum: 47.9 }),
            Rc::new(RoundingQuoteThresholdAdjuster { step: 0.1 }),
            Rc::new(FlaggedQuoteThresholdAdjuster {
                flags: flags.clone(),
                flag: "quote-threshold-legacy".to_string(),
                percent: -9.3,
            }),
            Rc::new(LocaleQuoteThresholdAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.1)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 11922.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_threshold_engine(&flags::new_registry());
        let item = new_quote_threshold("code-860", "title-302");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
