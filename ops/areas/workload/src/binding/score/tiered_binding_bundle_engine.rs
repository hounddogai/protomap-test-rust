use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered binding bundle calculation.
pub trait TieredBindingBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredBindingBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredBindingBundleAdjuster {
    pub minimum: f64,
}

impl TieredBindingBundleAdjuster for MinimumTieredBindingBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredBindingBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredBindingBundleAdjuster {
    pub cap: f64,
}

impl TieredBindingBundleAdjuster for CapTieredBindingBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredBindingBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingBundleQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl TieredBindingBundleAdjuster for TieredBindingBundleQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &TieredBindingBundle, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the tiered binding bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredBindingBundleEngine {
    pub steps: Vec<Rc<dyn TieredBindingBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredBindingBundleEngine {
    pub fn compute(&mut self, item: &TieredBindingBundle) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredBindingBundleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredBindingBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_binding_bundle_engine() -> TieredBindingBundleEngine {
    TieredBindingBundleEngine {
        steps: vec![
            Rc::new(MinimumTieredBindingBundleAdjuster { minimum: 9.7 }) as Rc<dyn TieredBindingBundleAdjuster>,
            Rc::new(CapTieredBindingBundleAdjuster { cap: 23336.0 }),
            Rc::new(TieredBindingBundleQuotaUnitsOffsetAdjuster { weight: 1.95 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 16235.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_binding_bundle_engine();
        let item = new_tiered_binding_bundle("code-666", "title-201");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
