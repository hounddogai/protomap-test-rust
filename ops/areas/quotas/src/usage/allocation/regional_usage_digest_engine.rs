use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional usage digest calculation.
pub trait RegionalUsageDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalUsageDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalUsageDigestAdjuster {
    pub step: f64,
}

impl RegionalUsageDigestModifier for RoundingRegionalUsageDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalUsageDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalUsageDigestAdjuster {
    pub cap: f64,
}

impl RegionalUsageDigestModifier for CapRegionalUsageDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalUsageDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalUsageDigestAdjuster {
    pub minimum: f64,
}

impl RegionalUsageDigestModifier for MinimumRegionalUsageDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalUsageDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalUsageDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalUsageDigestModifier for TieredRegionalUsageDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalUsageDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalUsageDigestAdjuster {
    pub percent: f64,
}

impl RegionalUsageDigestModifier for PercentRegionalUsageDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalUsageDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional usage digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalUsageDigestEngine {
    pub steps: Vec<Rc<dyn RegionalUsageDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalUsageDigestEngine {
    pub fn compute(&mut self, item: &RegionalUsageDigest) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalUsageDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_usage_digest_engine() -> RegionalUsageDigestEngine {
    RegionalUsageDigestEngine {
        steps: vec![
            Rc::new(RoundingRegionalUsageDigestAdjuster { step: 1.0 }) as Rc<dyn RegionalUsageDigestModifier>,
            Rc::new(CapRegionalUsageDigestAdjuster { cap: 30735.0 }),
            Rc::new(MinimumRegionalUsageDigestAdjuster { minimum: 35.6 }),
            Rc::new(TieredRegionalUsageDigestAdjuster {
                bounds: vec![1744.0, 3254.0, 9054.0],
                rates: vec![0.86, 0.9, 0.9],
            }),
            Rc::new(PercentRegionalUsageDigestAdjuster { percent: -2.5 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 54470.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_usage_digest_engine();
        let item = new_regional_usage_digest("code-727", "label-484");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
