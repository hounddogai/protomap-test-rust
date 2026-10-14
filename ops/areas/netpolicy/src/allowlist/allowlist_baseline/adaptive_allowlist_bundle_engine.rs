use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive allowlist bundle calculation.
pub trait AdaptiveAllowlistBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveAllowlistBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBundleCeilingOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveAllowlistBundleModifier for AdaptiveAllowlistBundleCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveAllowlistBundle, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveAllowlistBundleAdjuster {
    pub cap: f64,
}

impl AdaptiveAllowlistBundleModifier for CapAdaptiveAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveAllowlistBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveAllowlistBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveAllowlistBundleModifier for FlaggedAdaptiveAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveAllowlistBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveAllowlistBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveAllowlistBundleModifier for TieredAdaptiveAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveAllowlistBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveAllowlistBundleAdjuster {
    pub step: f64,
}

impl AdaptiveAllowlistBundleModifier for RoundingAdaptiveAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveAllowlistBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveAllowlistBundleAdjuster {
    pub percent: f64,
}

impl AdaptiveAllowlistBundleModifier for PercentAdaptiveAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveAllowlistBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive allowlist bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveAllowlistBundleEngine {
    pub steps: Vec<Rc<dyn AdaptiveAllowlistBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveAllowlistBundleEngine {
    pub fn compute(&mut self, item: &AdaptiveAllowlistBundle) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveAllowlistBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveAllowlistBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_allowlist_bundle_engine(flags: &flags::Registry) -> AdaptiveAllowlistBundleEngine {
    AdaptiveAllowlistBundleEngine {
        steps: vec![
            Rc::new(AdaptiveAllowlistBundleCeilingOffsetAdjuster { weight: 0.51 })
                as Rc<dyn AdaptiveAllowlistBundleModifier>,
            Rc::new(CapAdaptiveAllowlistBundleAdjuster { cap: 33961.0 }),
            Rc::new(FlaggedAdaptiveAllowlistBundleAdjuster {
                flags: flags.clone(),
                flag: "adaptive-allowlist-bundle-weighted".to_string(),
                percent: -8.1,
            }),
            Rc::new(TieredAdaptiveAllowlistBundleAdjuster {
                bounds: vec![7185.0, 8360.0],
                rates: vec![1.17, 1.02],
            }),
            Rc::new(RoundingAdaptiveAllowlistBundleAdjuster { step: 0.25 }),
            Rc::new(PercentAdaptiveAllowlistBundleAdjuster { percent: -23.9 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 87219.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_allowlist_bundle_engine(&flags::new_registry());
        let item = new_adaptive_allowlist_bundle("code-468", "name-629");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
