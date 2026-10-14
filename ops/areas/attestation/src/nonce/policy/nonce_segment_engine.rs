use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce segment calculation.
pub trait NonceSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceSegmentAdjuster {
    pub minimum: f64,
}

impl NonceSegmentModifier for MinimumNonceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSegmentCeilingOffsetAdjuster {
    pub weight: f64,
}

impl NonceSegmentModifier for NonceSegmentCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &NonceSegment, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceSegmentModifier for TieredNonceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceSegmentAdjuster {
    pub step: f64,
}

impl NonceSegmentModifier for RoundingNonceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceSegmentModifier for FlaggedNonceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the nonce segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceSegmentEngine {
    pub steps: Vec<Rc<dyn NonceSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceSegmentEngine {
    pub fn compute(&mut self, item: &NonceSegment) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[NonceSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_nonce_segment_engine(flags: &flags::Registry) -> NonceSegmentEngine {
    NonceSegmentEngine {
        steps: vec![
            Rc::new(MinimumNonceSegmentAdjuster { minimum: 39.9 }) as Rc<dyn NonceSegmentModifier>,
            Rc::new(NonceSegmentCeilingOffsetAdjuster { weight: 0.2 }),
            Rc::new(TieredNonceSegmentAdjuster {
                bounds: vec![3004.0, 3823.0, 6647.0],
                rates: vec![1.04, 1.16, 1.03],
            }),
            Rc::new(RoundingNonceSegmentAdjuster { step: 0.1 }),
            Rc::new(FlaggedNonceSegmentAdjuster {
                flags: flags.clone(),
                flag: "nonce-segment-legacy".to_string(),
                percent: 2.8,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 41611.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_segment_engine(&flags::new_registry());
        let item = new_nonce_segment("code-980", "name-986");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
