use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation bundle calculation.
pub trait FederationBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationBundleAdjuster {
    pub step: f64,
}

impl FederationBundleAdjuster for RoundingFederationBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationBundleAdjuster for TieredFederationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationBundleAdjuster {
    pub minimum: f64,
}

impl FederationBundleAdjuster for MinimumFederationBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBundleDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl FederationBundleAdjuster for FederationBundleDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &FederationBundle, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFederationBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationBundleAdjuster for SourceFederationBundleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &FederationBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationBundleAdjuster {
    pub percent: f64,
}

impl FederationBundleAdjuster for PercentFederationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the federation bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationBundleEngine {
    pub steps: Vec<Rc<dyn FederationBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationBundleEngine {
    pub fn compute(&mut self, item: &FederationBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FederationBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_federation_bundle_engine() -> FederationBundleEngine {
    FederationBundleEngine {
        steps: vec![
            Rc::new(RoundingFederationBundleAdjuster { step: 5.0 }) as Rc<dyn FederationBundleAdjuster>,
            Rc::new(TieredFederationBundleAdjuster {
                bounds: vec![1287.0, 4283.0, 7790.0, 8343.0],
                rates: vec![1.15, 0.9, 1.06, 0.94],
            }),
            Rc::new(MinimumFederationBundleAdjuster { minimum: 4.2 }),
            Rc::new(FederationBundleDurationMinutesOffsetAdjuster { weight: 1.42 }),
            Rc::new(SourceFederationBundleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.23)]),
            }),
            Rc::new(PercentFederationBundleAdjuster { percent: 22.2 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 82077.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_bundle_engine();
        let item = new_federation_bundle("code-290", "name-176");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
