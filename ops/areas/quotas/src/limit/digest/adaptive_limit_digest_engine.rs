use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive limit digest calculation.
pub trait AdaptiveLimitDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveLimitDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitDigestBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveLimitDigestModifier for AdaptiveLimitDigestBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveLimitDigest, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveLimitDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveLimitDigestModifier for TieredAdaptiveLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveLimitDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveLimitDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveLimitDigestModifier for FlaggedAdaptiveLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveLimitDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveLimitDigestAdjuster {
    pub percent: f64,
}

impl AdaptiveLimitDigestModifier for PercentAdaptiveLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveLimitDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveLimitDigestAdjuster {
    pub cap: f64,
}

impl AdaptiveLimitDigestModifier for CapAdaptiveLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveLimitDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveLimitDigestAdjuster {
    pub minimum: f64,
}

impl AdaptiveLimitDigestModifier for MinimumAdaptiveLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveLimitDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive limit digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveLimitDigestEngine {
    pub steps: Vec<Rc<dyn AdaptiveLimitDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveLimitDigestEngine {
    pub fn evaluate(&mut self, item: &AdaptiveLimitDigest) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveLimitDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_limit_digest_engine(flags: &flags::Registry) -> AdaptiveLimitDigestEngine {
    AdaptiveLimitDigestEngine {
        steps: vec![
            Rc::new(AdaptiveLimitDigestBatchSizeOffsetAdjuster { weight: 0.63 }) as Rc<dyn AdaptiveLimitDigestModifier>,
            Rc::new(TieredAdaptiveLimitDigestAdjuster {
                bounds: vec![1177.0, 1869.0, 4793.0, 8168.0],
                rates: vec![1.11, 0.99, 1.01, 0.95],
            }),
            Rc::new(FlaggedAdaptiveLimitDigestAdjuster {
                flags: flags.clone(),
                flag: "adaptive-limit-digest-pending".to_string(),
                percent: -7.8,
            }),
            Rc::new(PercentAdaptiveLimitDigestAdjuster { percent: 17.2 }),
            Rc::new(CapAdaptiveLimitDigestAdjuster { cap: 47302.0 }),
            Rc::new(MinimumAdaptiveLimitDigestAdjuster { minimum: 2.2 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 79465.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_limit_digest_engine(&flags::new_registry());
        let item = new_adaptive_limit_digest("code-85", "title-321");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
