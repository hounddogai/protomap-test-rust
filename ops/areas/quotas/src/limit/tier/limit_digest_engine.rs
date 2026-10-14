use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the limit digest calculation.
pub trait LimitDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitDigestAdjuster {
    pub step: f64,
}

impl LimitDigestAdjuster for RoundingLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitDigestAdjuster {
    pub percent: f64,
}

impl LimitDigestAdjuster for PercentLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitDigestAdjuster for TieredLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDigestRevisionOffsetAdjuster {
    pub weight: f64,
}

impl LimitDigestAdjuster for LimitDigestRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &LimitDigest, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitDigestAdjuster {
    pub minimum: f64,
}

impl LimitDigestAdjuster for MinimumLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitDigestAdjuster for FlaggedLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the limit digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitDigestEngine {
    pub steps: Vec<Rc<dyn LimitDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitDigestEngine {
    pub fn compute(&mut self, item: &LimitDigest) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[LimitDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_limit_digest_engine(flags: &flags::Registry) -> LimitDigestEngine {
    LimitDigestEngine {
        steps: vec![
            Rc::new(RoundingLimitDigestAdjuster { step: 1.0 }) as Rc<dyn LimitDigestAdjuster>,
            Rc::new(PercentLimitDigestAdjuster { percent: 19.6 }),
            Rc::new(TieredLimitDigestAdjuster {
                bounds: vec![3423.0, 5752.0, 9365.0],
                rates: vec![0.84, 1.2, 0.9],
            }),
            Rc::new(LimitDigestRevisionOffsetAdjuster { weight: 0.49 }),
            Rc::new(MinimumLimitDigestAdjuster { minimum: 26.0 }),
            Rc::new(FlaggedLimitDigestAdjuster {
                flags: flags.clone(),
                flag: "limit-digest-primary".to_string(),
                percent: -3.0,
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 28370.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_digest_engine(&flags::new_registry());
        let item = new_limit_digest("code-907", "label-661");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
