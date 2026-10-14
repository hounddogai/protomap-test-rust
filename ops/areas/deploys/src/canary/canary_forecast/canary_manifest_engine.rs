use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary manifest calculation.
pub trait CanaryManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryManifestModifier for TieredCanaryManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryManifestThresholdOffsetAdjuster {
    pub weight: f64,
}

impl CanaryManifestModifier for CanaryManifestThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryManifest, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryManifestAdjuster {
    pub percent: f64,
}

impl CanaryManifestModifier for PercentCanaryManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryManifestAdjuster {
    pub step: f64,
}

impl CanaryManifestModifier for RoundingCanaryManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryManifestAdjuster {
    pub cap: f64,
}

impl CanaryManifestModifier for CapCanaryManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the canary manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryManifestEngine {
    pub steps: Vec<Rc<dyn CanaryManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryManifestEngine {
    pub fn compute(&mut self, item: &CanaryManifest) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[CanaryManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_canary_manifest_engine() -> CanaryManifestEngine {
    CanaryManifestEngine {
        steps: vec![
            Rc::new(TieredCanaryManifestAdjuster {
                bounds: vec![777.0, 7888.0, 8746.0, 9626.0],
                rates: vec![1.12, 1.02, 1.19, 0.95],
            }) as Rc<dyn CanaryManifestModifier>,
            Rc::new(CanaryManifestThresholdOffsetAdjuster { weight: 1.79 }),
            Rc::new(PercentCanaryManifestAdjuster { percent: 6.4 }),
            Rc::new(RoundingCanaryManifestAdjuster { step: 0.5 }),
            Rc::new(CapCanaryManifestAdjuster { cap: 3350.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 15067.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_manifest_engine();
        let item = new_canary_manifest("code-607", "name-695");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
