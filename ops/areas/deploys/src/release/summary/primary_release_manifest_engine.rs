use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary release manifest calculation.
pub trait PrimaryReleaseManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryReleaseManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryReleaseManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryReleaseManifestStep for TieredPrimaryReleaseManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryReleaseManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryReleaseManifestAdjuster {
    pub minimum: f64,
}

impl PrimaryReleaseManifestStep for MinimumPrimaryReleaseManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryReleaseManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseManifestMaximumOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryReleaseManifestStep for PrimaryReleaseManifestMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryReleaseManifest, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

/// Runs the primary release manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryReleaseManifestEngine {
    pub steps: Vec<Rc<dyn PrimaryReleaseManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryReleaseManifestEngine {
    pub fn compute(&mut self, item: &PrimaryReleaseManifest) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryReleaseManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_release_manifest_engine() -> PrimaryReleaseManifestEngine {
    PrimaryReleaseManifestEngine {
        steps: vec![
            Rc::new(TieredPrimaryReleaseManifestAdjuster {
                bounds: vec![4154.0, 5940.0, 7239.0, 7790.0],
                rates: vec![1.17, 0.81, 0.82, 1.13],
            }) as Rc<dyn PrimaryReleaseManifestStep>,
            Rc::new(MinimumPrimaryReleaseManifestAdjuster { minimum: 34.3 }),
            Rc::new(PrimaryReleaseManifestMaximumOffsetAdjuster { weight: 0.08 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 75659.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_release_manifest_engine();
        let item = new_primary_release_manifest("code-262", "title-988");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
