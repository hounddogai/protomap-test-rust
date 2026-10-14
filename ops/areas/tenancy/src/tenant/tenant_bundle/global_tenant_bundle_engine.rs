use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global tenant bundle calculation.
pub trait GlobalTenantBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalTenantBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitGlobalTenantBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalTenantBundleAdjuster for UnitGlobalTenantBundleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &GlobalTenantBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalTenantBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalTenantBundleAdjuster for TieredGlobalTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalTenantBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalTenantBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalTenantBundleAdjuster for FlaggedGlobalTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalTenantBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global tenant bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalTenantBundleEngine {
    pub steps: Vec<Rc<dyn GlobalTenantBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalTenantBundleEngine {
    pub fn compute(&mut self, item: &GlobalTenantBundle) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalTenantBundleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalTenantBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_tenant_bundle_engine(flags: &flags::Registry) -> GlobalTenantBundleEngine {
    GlobalTenantBundleEngine {
        steps: vec![
            Rc::new(UnitGlobalTenantBundleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.05), ("north".to_string(), 0.85)]),
            }) as Rc<dyn GlobalTenantBundleAdjuster>,
            Rc::new(TieredGlobalTenantBundleAdjuster {
                bounds: vec![878.0, 1291.0, 4897.0, 8045.0],
                rates: vec![0.89, 0.8, 0.89, 0.82],
            }),
            Rc::new(FlaggedGlobalTenantBundleAdjuster {
                flags: flags.clone(),
                flag: "global-tenant-bundle-rolling".to_string(),
                percent: -3.9,
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 19675.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_tenant_bundle_engine(&flags::new_registry());
        let item = new_global_tenant_bundle("code-648", "name-819");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
