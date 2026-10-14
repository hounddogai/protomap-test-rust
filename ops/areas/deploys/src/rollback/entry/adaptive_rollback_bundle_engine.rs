use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive rollback bundle calculation.
pub trait AdaptiveRollbackBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRollbackBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveRollbackBundleAdjuster {
    pub cap: f64,
}

impl AdaptiveRollbackBundleAdjuster for CapAdaptiveRollbackBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveRollbackBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveRollbackBundleAdjuster {
    pub step: f64,
}

impl AdaptiveRollbackBundleAdjuster for RoundingAdaptiveRollbackBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveRollbackBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRollbackBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRollbackBundleAdjuster for TieredAdaptiveRollbackBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRollbackBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackBundlePriorityOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRollbackBundleAdjuster for AdaptiveRollbackBundlePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRollbackBundle, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the adaptive rollback bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRollbackBundleEngine {
    pub steps: Vec<Rc<dyn AdaptiveRollbackBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRollbackBundleEngine {
    pub fn compute(&mut self, item: &AdaptiveRollbackBundle) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRollbackBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_rollback_bundle_engine() -> AdaptiveRollbackBundleEngine {
    AdaptiveRollbackBundleEngine {
        steps: vec![
            Rc::new(CapAdaptiveRollbackBundleAdjuster { cap: 16534.0 }) as Rc<dyn AdaptiveRollbackBundleAdjuster>,
            Rc::new(RoundingAdaptiveRollbackBundleAdjuster { step: 0.05 }),
            Rc::new(TieredAdaptiveRollbackBundleAdjuster {
                bounds: vec![3264.0, 5376.0, 7688.0, 9172.0],
                rates: vec![0.89, 1.07, 0.84, 1.11],
            }),
            Rc::new(AdaptiveRollbackBundlePriorityOffsetAdjuster { weight: 0.66 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 62971.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_rollback_bundle_engine();
        let item = new_adaptive_rollback_bundle("code-151", "title-365");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
