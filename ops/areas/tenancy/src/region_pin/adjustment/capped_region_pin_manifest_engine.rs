use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped region pin manifest calculation.
pub trait CappedRegionPinManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRegionPinManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedRegionPinManifestAdjuster {
    pub percent: f64,
}

impl CappedRegionPinManifestFactor for PercentCappedRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedRegionPinManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedRegionPinManifestAdjuster {
    pub step: f64,
}

impl CappedRegionPinManifestFactor for RoundingCappedRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedRegionPinManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedRegionPinManifestAdjuster {
    pub minimum: f64,
}

impl CappedRegionPinManifestFactor for MinimumCappedRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedRegionPinManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedRegionPinManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedRegionPinManifestFactor for TieredCappedRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedRegionPinManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinManifestSequenceOffsetAdjuster {
    pub weight: f64,
}

impl CappedRegionPinManifestFactor for CappedRegionPinManifestSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &CappedRegionPinManifest, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the capped region pin manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRegionPinManifestEngine {
    pub steps: Vec<Rc<dyn CappedRegionPinManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRegionPinManifestEngine {
    pub fn compute(&mut self, item: &CappedRegionPinManifest) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedRegionPinManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedRegionPinManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_region_pin_manifest_engine() -> CappedRegionPinManifestEngine {
    CappedRegionPinManifestEngine {
        steps: vec![
            Rc::new(PercentCappedRegionPinManifestAdjuster { percent: 23.3 }) as Rc<dyn CappedRegionPinManifestFactor>,
            Rc::new(RoundingCappedRegionPinManifestAdjuster { step: 5.0 }),
            Rc::new(MinimumCappedRegionPinManifestAdjuster { minimum: 12.2 }),
            Rc::new(TieredCappedRegionPinManifestAdjuster {
                bounds: vec![3636.0, 4563.0],
                rates: vec![0.81, 1.18],
            }),
            Rc::new(CappedRegionPinManifestSequenceOffsetAdjuster { weight: 0.59 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 84835.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_region_pin_manifest_engine();
        let item = new_capped_region_pin_manifest("code-805", "name-503");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
