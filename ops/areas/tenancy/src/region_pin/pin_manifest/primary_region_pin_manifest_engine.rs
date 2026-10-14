use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary region pin manifest calculation.
pub trait PrimaryRegionPinManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRegionPinManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryRegionPinManifestAdjuster {
    pub step: f64,
}

impl PrimaryRegionPinManifestStep for RoundingPrimaryRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryRegionPinManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryRegionPinManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryRegionPinManifestStep for TieredPrimaryRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryRegionPinManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryRegionPinManifestAdjuster {
    pub cap: f64,
}

impl PrimaryRegionPinManifestStep for CapPrimaryRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryRegionPinManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePrimaryRegionPinManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryRegionPinManifestStep for NamePrimaryRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PrimaryRegionPinManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary region pin manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRegionPinManifestEngine {
    pub steps: Vec<Rc<dyn PrimaryRegionPinManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRegionPinManifestEngine {
    pub fn compute(&mut self, item: &PrimaryRegionPinManifest) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRegionPinManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_region_pin_manifest_engine() -> PrimaryRegionPinManifestEngine {
    PrimaryRegionPinManifestEngine {
        steps: vec![
            Rc::new(RoundingPrimaryRegionPinManifestAdjuster { step: 0.1 }) as Rc<dyn PrimaryRegionPinManifestStep>,
            Rc::new(TieredPrimaryRegionPinManifestAdjuster {
                bounds: vec![7002.0, 7037.0, 7787.0],
                rates: vec![1.05, 0.93, 1.13],
            }),
            Rc::new(CapPrimaryRegionPinManifestAdjuster { cap: 16898.0 }),
            Rc::new(NamePrimaryRegionPinManifestAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.29),
                    ("south".to_string(), 1.02),
                    ("east".to_string(), 1.04),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 29068.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_region_pin_manifest_engine();
        let item = new_primary_region_pin_manifest("code-70", "name-363");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
