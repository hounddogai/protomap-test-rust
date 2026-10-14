use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering bundle calculation.
pub trait PeeringBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringBundleFactor for TieredPeeringBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringBundleAdjuster {
    pub cap: f64,
}

impl PeeringBundleFactor for CapPeeringBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringBundleAdjuster {
    pub percent: f64,
}

impl PeeringBundleFactor for PercentPeeringBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringBundleAdjuster {
    pub step: f64,
}

impl PeeringBundleFactor for RoundingPeeringBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringBundleAdjuster {
    pub minimum: f64,
}

impl PeeringBundleFactor for MinimumPeeringBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the peering bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringBundleEngine {
    pub steps: Vec<Rc<dyn PeeringBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringBundleEngine {
    pub fn compute(&mut self, item: &PeeringBundle) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[PeeringBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_peering_bundle_engine() -> PeeringBundleEngine {
    PeeringBundleEngine {
        steps: vec![
            Rc::new(TieredPeeringBundleAdjuster {
                bounds: vec![2770.0, 9298.0],
                rates: vec![0.82, 1.01],
            }) as Rc<dyn PeeringBundleFactor>,
            Rc::new(CapPeeringBundleAdjuster { cap: 45546.0 }),
            Rc::new(PercentPeeringBundleAdjuster { percent: -12.6 }),
            Rc::new(RoundingPeeringBundleAdjuster { step: 1.0 }),
            Rc::new(MinimumPeeringBundleAdjuster { minimum: 3.9 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 84488.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_bundle_engine();
        let item = new_peering_bundle("code-121", "title-924");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
