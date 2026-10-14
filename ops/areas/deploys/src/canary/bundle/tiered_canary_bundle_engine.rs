use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered canary bundle calculation.
pub trait TieredCanaryBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredCanaryBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredCanaryBundleAdjuster {
    pub minimum: f64,
}

impl TieredCanaryBundleModifier for MinimumTieredCanaryBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredCanaryBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredCanaryBundleAdjuster {
    pub cap: f64,
}

impl TieredCanaryBundleModifier for CapTieredCanaryBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredCanaryBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredCanaryBundleAdjuster {
    pub step: f64,
}

impl TieredCanaryBundleModifier for RoundingTieredCanaryBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredCanaryBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryBundleExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl TieredCanaryBundleModifier for TieredCanaryBundleExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &TieredCanaryBundle, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredCanaryBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredCanaryBundleModifier for TieredTieredCanaryBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredCanaryBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTieredCanaryBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredCanaryBundleModifier for NameTieredCanaryBundleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TieredCanaryBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tiered canary bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredCanaryBundleEngine {
    pub steps: Vec<Rc<dyn TieredCanaryBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredCanaryBundleEngine {
    pub fn run(&mut self, item: &TieredCanaryBundle) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TieredCanaryBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_canary_bundle_engine() -> TieredCanaryBundleEngine {
    TieredCanaryBundleEngine {
        steps: vec![
            Rc::new(MinimumTieredCanaryBundleAdjuster { minimum: 41.7 }) as Rc<dyn TieredCanaryBundleModifier>,
            Rc::new(CapTieredCanaryBundleAdjuster { cap: 31767.0 }),
            Rc::new(RoundingTieredCanaryBundleAdjuster { step: 5.0 }),
            Rc::new(TieredCanaryBundleExpiresAtOffsetAdjuster { weight: 1.88 }),
            Rc::new(TieredTieredCanaryBundleAdjuster {
                bounds: vec![4394.0, 9208.0],
                rates: vec![1.01, 1.09],
            }),
            Rc::new(NameTieredCanaryBundleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.27), ("west".to_string(), 0.88)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 4594.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_canary_bundle_engine();
        let item = new_tiered_canary_bundle("code-379", "name-769");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
