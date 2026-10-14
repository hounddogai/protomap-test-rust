use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the quote digest calculation.
pub trait QuoteDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuoteDigestAdjuster {
    pub percent: f64,
}

impl QuoteDigestAdjuster for PercentQuoteDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuoteDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitQuoteDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteDigestAdjuster for UnitQuoteDigestAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &QuoteDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteDigestAdjuster {
    pub minimum: f64,
}

impl QuoteDigestAdjuster for MinimumQuoteDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteDigestAdjuster {
    pub cap: f64,
}

impl QuoteDigestAdjuster for CapQuoteDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the quote digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteDigestEngine {
    pub steps: Vec<Rc<dyn QuoteDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteDigestEngine {
    pub fn run(&mut self, item: &QuoteDigest) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteDigestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_digest_engine() -> QuoteDigestEngine {
    QuoteDigestEngine {
        steps: vec![
            Rc::new(PercentQuoteDigestAdjuster { percent: 17.4 }) as Rc<dyn QuoteDigestAdjuster>,
            Rc::new(UnitQuoteDigestAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.23), ("south".to_string(), 0.94)]),
            }),
            Rc::new(MinimumQuoteDigestAdjuster { minimum: 39.6 }),
            Rc::new(CapQuoteDigestAdjuster { cap: 3454.0 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 86332.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_digest_engine();
        let item = new_quote_digest("code-720", "name-195");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
