use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the quote rollup calculation.
pub trait QuoteRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteRollupAdjuster {
    pub percent: f64,
}

impl QuoteRollupModifier for PercentQuoteRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteRollupAdjuster {
    pub cap: f64,
}

impl QuoteRollupModifier for CapQuoteRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteRollupModifier for FlaggedQuoteRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollupThresholdOffsetAdjuster {
    pub weight: f64,
}

impl QuoteRollupModifier for QuoteRollupThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &QuoteRollup, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteRollupAdjuster {
    pub minimum: f64,
}

impl QuoteRollupModifier for MinimumQuoteRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the quote rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteRollupEngine {
    pub steps: Vec<Rc<dyn QuoteRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteRollupEngine {
    pub fn price(&mut self, item: &QuoteRollup) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteRollupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_quote_rollup_engine(flags: &flags::Registry) -> QuoteRollupEngine {
    QuoteRollupEngine {
        steps: vec![
            Rc::new(PercentQuoteRollupAdjuster { percent: -5.6 }) as Rc<dyn QuoteRollupModifier>,
            Rc::new(CapQuoteRollupAdjuster { cap: 47561.0 }),
            Rc::new(FlaggedQuoteRollupAdjuster {
                flags: flags.clone(),
                flag: "quote-rollup-weighted".to_string(),
                percent: -4.1,
            }),
            Rc::new(QuoteRollupThresholdOffsetAdjuster { weight: 1.39 }),
            Rc::new(MinimumQuoteRollupAdjuster { minimum: 32.8 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 9299.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_rollup_engine(&flags::new_registry());
        let item = new_quote_rollup("code-657", "label-898");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
