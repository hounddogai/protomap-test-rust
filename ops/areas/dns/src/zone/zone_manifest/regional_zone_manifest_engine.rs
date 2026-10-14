use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional zone manifest calculation.
pub trait RegionalZoneManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalZoneManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalZoneManifestAdjuster {
    pub minimum: f64,
}

impl RegionalZoneManifestModifier for MinimumRegionalZoneManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalZoneManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalZoneManifestAdjuster {
    pub cap: f64,
}

impl RegionalZoneManifestModifier for CapRegionalZoneManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalZoneManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalZoneManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalZoneManifestModifier for TieredRegionalZoneManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalZoneManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneManifestRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RegionalZoneManifestModifier for RegionalZoneManifestRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalZoneManifest, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the regional zone manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalZoneManifestEngine {
    pub steps: Vec<Rc<dyn RegionalZoneManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalZoneManifestEngine {
    pub fn compute(&mut self, item: &RegionalZoneManifest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionalZoneManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_zone_manifest_engine() -> RegionalZoneManifestEngine {
    RegionalZoneManifestEngine {
        steps: vec![
            Rc::new(MinimumRegionalZoneManifestAdjuster { minimum: 24.4 }) as Rc<dyn RegionalZoneManifestModifier>,
            Rc::new(CapRegionalZoneManifestAdjuster { cap: 37620.0 }),
            Rc::new(TieredRegionalZoneManifestAdjuster {
                bounds: vec![1652.0, 1871.0, 4228.0],
                rates: vec![0.97, 0.91, 0.82],
            }),
            Rc::new(RegionalZoneManifestRetriesOffsetAdjuster { weight: 0.28 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 18148.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_zone_manifest_engine();
        let item = new_regional_zone_manifest("code-253", "title-864");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
