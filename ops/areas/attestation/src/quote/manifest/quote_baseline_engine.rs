use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the quote baseline calculation.
pub trait QuoteBaselineFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBaselineMarginOffsetAdjuster {
    pub weight: f64,
}

impl QuoteBaselineFactor for QuoteBaselineMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &QuoteBaseline, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteBaselineAdjuster {
    pub percent: f64,
}

impl QuoteBaselineFactor for PercentQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteBaselineAdjuster {
    pub cap: f64,
}

impl QuoteBaselineFactor for CapQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteBaselineAdjuster {
    pub minimum: f64,
}

impl QuoteBaselineFactor for MinimumQuoteBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the quote baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteBaselineEngine {
    pub steps: Vec<Rc<dyn QuoteBaselineFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteBaselineEngine {
    pub fn run(&mut self, item: &QuoteBaseline) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteBaselineFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_baseline_engine() -> QuoteBaselineEngine {
    QuoteBaselineEngine {
        steps: vec![
            Rc::new(QuoteBaselineMarginOffsetAdjuster { weight: 1.98 }) as Rc<dyn QuoteBaselineFactor>,
            Rc::new(PercentQuoteBaselineAdjuster { percent: 17.3 }),
            Rc::new(CapQuoteBaselineAdjuster { cap: 45019.0 }),
            Rc::new(MinimumQuoteBaselineAdjuster { minimum: 33.1 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 77321.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_baseline_engine();
        let item = new_quote_baseline("code-559", "title-843");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
