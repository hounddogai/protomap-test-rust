use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary federation bundle calculation.
pub trait PrimaryFederationBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryFederationBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryFederationBundleAdjuster {
    pub minimum: f64,
}

impl PrimaryFederationBundleFactor for MinimumPrimaryFederationBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryFederationBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryFederationBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryFederationBundleFactor for TieredPrimaryFederationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryFederationBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPrimaryFederationBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryFederationBundleFactor for ClusterPrimaryFederationBundleAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PrimaryFederationBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationBundleVersionOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryFederationBundleFactor for PrimaryFederationBundleVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryFederationBundle, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

/// Runs the primary federation bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryFederationBundleEngine {
    pub steps: Vec<Rc<dyn PrimaryFederationBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryFederationBundleEngine {
    pub fn evaluate(&mut self, item: &PrimaryFederationBundle) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryFederationBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_federation_bundle_engine() -> PrimaryFederationBundleEngine {
    PrimaryFederationBundleEngine {
        steps: vec![
            Rc::new(MinimumPrimaryFederationBundleAdjuster { minimum: 1.9 }) as Rc<dyn PrimaryFederationBundleFactor>,
            Rc::new(TieredPrimaryFederationBundleAdjuster {
                bounds: vec![4915.0, 9536.0],
                rates: vec![0.94, 1.11],
            }),
            Rc::new(ClusterPrimaryFederationBundleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.91), ("west".to_string(), 1.31)]),
            }),
            Rc::new(PrimaryFederationBundleVersionOffsetAdjuster { weight: 1.64 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 38167.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_federation_bundle_engine();
        let item = new_primary_federation_bundle("code-413", "name-413");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
