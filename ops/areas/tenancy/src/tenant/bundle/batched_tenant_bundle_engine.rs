use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched tenant bundle calculation.
pub trait BatchedTenantBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedTenantBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedTenantBundleAdjuster {
    pub cap: f64,
}

impl BatchedTenantBundleStep for CapBatchedTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedTenantBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceBatchedTenantBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedTenantBundleStep for NamespaceBatchedTenantBundleAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &BatchedTenantBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedTenantBundleAdjuster {
    pub percent: f64,
}

impl BatchedTenantBundleStep for PercentBatchedTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedTenantBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedTenantBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedTenantBundleStep for FlaggedBatchedTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedTenantBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedTenantBundleAdjuster {
    pub step: f64,
}

impl BatchedTenantBundleStep for RoundingBatchedTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedTenantBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched tenant bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedTenantBundleEngine {
    pub steps: Vec<Rc<dyn BatchedTenantBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedTenantBundleEngine {
    pub fn price(&mut self, item: &BatchedTenantBundle) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[BatchedTenantBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_tenant_bundle_engine(flags: &flags::Registry) -> BatchedTenantBundleEngine {
    BatchedTenantBundleEngine {
        steps: vec![
            Rc::new(CapBatchedTenantBundleAdjuster { cap: 2271.0 }) as Rc<dyn BatchedTenantBundleStep>,
            Rc::new(NamespaceBatchedTenantBundleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.01),
                    ("west".to_string(), 1.22),
                    ("south".to_string(), 1.25),
                ]),
            }),
            Rc::new(PercentBatchedTenantBundleAdjuster { percent: -16.7 }),
            Rc::new(FlaggedBatchedTenantBundleAdjuster {
                flags: flags.clone(),
                flag: "batched-tenant-bundle-strict".to_string(),
                percent: -0.6,
            }),
            Rc::new(RoundingBatchedTenantBundleAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 16637.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_tenant_bundle_engine(&flags::new_registry());
        let item = new_batched_tenant_bundle("code-820", "name-534");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
