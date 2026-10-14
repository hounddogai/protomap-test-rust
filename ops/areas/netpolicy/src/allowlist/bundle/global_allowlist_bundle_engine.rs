use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the global allowlist bundle calculation.
pub trait GlobalAllowlistBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalAllowlistBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalAllowlistBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalAllowlistBundleStep for TieredGlobalAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalAllowlistBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalAllowlistBundleAdjuster {
    pub cap: f64,
}

impl GlobalAllowlistBundleStep for CapGlobalAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalAllowlistBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAllowlistBundleExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalAllowlistBundleStep for GlobalAllowlistBundleExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalAllowlistBundle, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalAllowlistBundleAdjuster {
    pub percent: f64,
}

impl GlobalAllowlistBundleStep for PercentGlobalAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalAllowlistBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global allowlist bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalAllowlistBundleEngine {
    pub steps: Vec<Rc<dyn GlobalAllowlistBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalAllowlistBundleEngine {
    pub fn compute(&mut self, item: &GlobalAllowlistBundle) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalAllowlistBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalAllowlistBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_allowlist_bundle_engine() -> GlobalAllowlistBundleEngine {
    GlobalAllowlistBundleEngine {
        steps: vec![
            Rc::new(TieredGlobalAllowlistBundleAdjuster {
                bounds: vec![6310.0, 7367.0],
                rates: vec![1.05, 1.14],
            }) as Rc<dyn GlobalAllowlistBundleStep>,
            Rc::new(CapGlobalAllowlistBundleAdjuster { cap: 42126.0 }),
            Rc::new(GlobalAllowlistBundleExpiresAtOffsetAdjuster { weight: 0.44 }),
            Rc::new(PercentGlobalAllowlistBundleAdjuster { percent: 1.4 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 55035.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_allowlist_bundle_engine();
        let item = new_global_allowlist_bundle("code-95", "name-42");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
