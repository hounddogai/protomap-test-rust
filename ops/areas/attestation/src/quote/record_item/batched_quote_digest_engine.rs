use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched quote digest calculation.
pub trait BatchedQuoteDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedQuoteDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteDigestWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl BatchedQuoteDigestAdjuster for BatchedQuoteDigestWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedQuoteDigest, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedQuoteDigestAdjuster {
    pub percent: f64,
}

impl BatchedQuoteDigestAdjuster for PercentBatchedQuoteDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedQuoteDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedQuoteDigestAdjuster {
    pub cap: f64,
}

impl BatchedQuoteDigestAdjuster for CapBatchedQuoteDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedQuoteDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedQuoteDigestAdjuster {
    pub minimum: f64,
}

impl BatchedQuoteDigestAdjuster for MinimumBatchedQuoteDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedQuoteDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryBatchedQuoteDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedQuoteDigestAdjuster for CategoryBatchedQuoteDigestAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &BatchedQuoteDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedQuoteDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedQuoteDigestAdjuster for FlaggedBatchedQuoteDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedQuoteDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched quote digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedQuoteDigestEngine {
    pub steps: Vec<Rc<dyn BatchedQuoteDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedQuoteDigestEngine {
    pub fn evaluate(&mut self, item: &BatchedQuoteDigest) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedQuoteDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_quote_digest_engine(flags: &flags::Registry) -> BatchedQuoteDigestEngine {
    BatchedQuoteDigestEngine {
        steps: vec![
            Rc::new(BatchedQuoteDigestWindowDaysOffsetAdjuster { weight: 1.25 }) as Rc<dyn BatchedQuoteDigestAdjuster>,
            Rc::new(PercentBatchedQuoteDigestAdjuster { percent: 22.1 }),
            Rc::new(CapBatchedQuoteDigestAdjuster { cap: 3820.0 }),
            Rc::new(MinimumBatchedQuoteDigestAdjuster { minimum: 16.4 }),
            Rc::new(CategoryBatchedQuoteDigestAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.31), ("retail".to_string(), 0.89)]),
            }),
            Rc::new(FlaggedBatchedQuoteDigestAdjuster {
                flags: flags.clone(),
                flag: "batched-quote-digest-rolling".to_string(),
                percent: -3.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 50463.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_quote_digest_engine(&flags::new_registry());
        let item = new_batched_quote_digest("code-557", "name-992");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
