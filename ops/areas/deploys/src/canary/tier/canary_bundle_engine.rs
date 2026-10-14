use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary bundle calculation.
pub trait CanaryBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleCanaryBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryBundleFactor for LocaleCanaryBundleAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &CanaryBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBundleQuantityOffsetAdjuster {
    pub weight: f64,
}

impl CanaryBundleFactor for CanaryBundleQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryBundle, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryBundleAdjuster {
    pub cap: f64,
}

impl CanaryBundleFactor for CapCanaryBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the canary bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryBundleEngine {
    pub steps: Vec<Rc<dyn CanaryBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryBundleEngine {
    pub fn compute(&mut self, item: &CanaryBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CanaryBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_canary_bundle_engine() -> CanaryBundleEngine {
    CanaryBundleEngine {
        steps: vec![
            Rc::new(LocaleCanaryBundleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.32), ("north".to_string(), 0.84)]),
            }) as Rc<dyn CanaryBundleFactor>,
            Rc::new(CanaryBundleQuantityOffsetAdjuster { weight: 1.11 }),
            Rc::new(CapCanaryBundleAdjuster { cap: 11447.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 54821.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_bundle_engine();
        let item = new_canary_bundle("code-487", "name-362");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
