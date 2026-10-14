use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the quote tier calculation.
pub trait QuoteTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteTierAdjuster {
    pub cap: f64,
}

impl QuoteTierFactor for CapQuoteTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteTierAdjuster {
    pub step: f64,
}

impl QuoteTierFactor for RoundingQuoteTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteTierAdjuster {
    pub minimum: f64,
}

impl QuoteTierFactor for MinimumQuoteTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteTierFactor for TieredQuoteTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the quote tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteTierEngine {
    pub steps: Vec<Rc<dyn QuoteTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteTierEngine {
    pub fn evaluate(&mut self, item: &QuoteTier) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteTierFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_quote_tier_engine() -> QuoteTierEngine {
    QuoteTierEngine {
        steps: vec![
            Rc::new(CapQuoteTierAdjuster { cap: 25453.0 }) as Rc<dyn QuoteTierFactor>,
            Rc::new(RoundingQuoteTierAdjuster { step: 5.0 }),
            Rc::new(MinimumQuoteTierAdjuster { minimum: 33.5 }),
            Rc::new(TieredQuoteTierAdjuster {
                bounds: vec![2237.0, 2601.0, 3693.0, 4989.0],
                rates: vec![1.15, 0.84, 0.94, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 67460.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_tier_engine();
        let item = new_quote_tier("code-72", "title-822");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
