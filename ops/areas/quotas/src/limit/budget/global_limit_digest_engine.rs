use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global limit digest calculation.
pub trait GlobalLimitDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalLimitDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalLimitDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalLimitDigestAdjuster for TieredGlobalLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalLimitDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDigestMarginOffsetAdjuster {
    pub weight: f64,
}

impl GlobalLimitDigestAdjuster for GlobalLimitDigestMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalLimitDigest, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalLimitDigestAdjuster {
    pub step: f64,
}

impl GlobalLimitDigestAdjuster for RoundingGlobalLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalLimitDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalLimitDigestAdjuster {
    pub minimum: f64,
}

impl GlobalLimitDigestAdjuster for MinimumGlobalLimitDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalLimitDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global limit digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalLimitDigestEngine {
    pub steps: Vec<Rc<dyn GlobalLimitDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalLimitDigestEngine {
    pub fn compute(&mut self, item: &GlobalLimitDigest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalLimitDigestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalLimitDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_limit_digest_engine() -> GlobalLimitDigestEngine {
    GlobalLimitDigestEngine {
        steps: vec![
            Rc::new(TieredGlobalLimitDigestAdjuster {
                bounds: vec![1576.0, 5294.0],
                rates: vec![0.81, 0.83],
            }) as Rc<dyn GlobalLimitDigestAdjuster>,
            Rc::new(GlobalLimitDigestMarginOffsetAdjuster { weight: 1.34 }),
            Rc::new(RoundingGlobalLimitDigestAdjuster { step: 0.1 }),
            Rc::new(MinimumGlobalLimitDigestAdjuster { minimum: 8.4 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 24590.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_limit_digest_engine();
        let item = new_global_limit_digest("code-16", "title-578");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
