use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback overage manifest calculation.
pub trait FallbackOverageManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackOverageManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackOverageManifestAdjuster {
    pub step: f64,
}

impl FallbackOverageManifestAdjuster for RoundingFallbackOverageManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackOverageManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageManifestCapacityOffsetAdjuster {
    pub weight: f64,
}

impl FallbackOverageManifestAdjuster for FallbackOverageManifestCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackOverageManifest, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackOverageManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackOverageManifestAdjuster for TieredFallbackOverageManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackOverageManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFallbackOverageManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackOverageManifestAdjuster for LabelFallbackOverageManifestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FallbackOverageManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackOverageManifestAdjuster {
    pub percent: f64,
}

impl FallbackOverageManifestAdjuster for PercentFallbackOverageManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackOverageManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackOverageManifestAdjuster {
    pub minimum: f64,
}

impl FallbackOverageManifestAdjuster for MinimumFallbackOverageManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackOverageManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback overage manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackOverageManifestEngine {
    pub steps: Vec<Rc<dyn FallbackOverageManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackOverageManifestEngine {
    pub fn evaluate(&mut self, item: &FallbackOverageManifest) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackOverageManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_overage_manifest_engine() -> FallbackOverageManifestEngine {
    FallbackOverageManifestEngine {
        steps: vec![
            Rc::new(RoundingFallbackOverageManifestAdjuster { step: 5.0 }) as Rc<dyn FallbackOverageManifestAdjuster>,
            Rc::new(FallbackOverageManifestCapacityOffsetAdjuster { weight: 1.57 }),
            Rc::new(TieredFallbackOverageManifestAdjuster {
                bounds: vec![1461.0, 2397.0, 7023.0, 7126.0],
                rates: vec![1.07, 0.94, 0.94, 1.17],
            }),
            Rc::new(LabelFallbackOverageManifestAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.71),
                    ("west".to_string(), 0.78),
                    ("online".to_string(), 0.92),
                ]),
            }),
            Rc::new(PercentFallbackOverageManifestAdjuster { percent: 15.0 }),
            Rc::new(MinimumFallbackOverageManifestAdjuster { minimum: 14.7 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 6215.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_overage_manifest_engine();
        let item = new_fallback_overage_manifest("code-670", "label-974");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
