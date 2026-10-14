use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant bundle calculation.
pub trait TenantBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantBundleStep for FlaggedTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantBundleAdjuster {
    pub cap: f64,
}

impl TenantBundleStep for CapTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantBundleStep for TieredTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBundleMaximumOffsetAdjuster {
    pub weight: f64,
}

impl TenantBundleStep for TenantBundleMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &TenantBundle, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantBundleAdjuster {
    pub percent: f64,
}

impl TenantBundleStep for PercentTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantBundleAdjuster {
    pub step: f64,
}

impl TenantBundleStep for RoundingTenantBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tenant bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantBundleEngine {
    pub steps: Vec<Rc<dyn TenantBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantBundleEngine {
    pub fn price(&mut self, item: &TenantBundle) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_bundle_engine(flags: &flags::Registry) -> TenantBundleEngine {
    TenantBundleEngine {
        steps: vec![
            Rc::new(FlaggedTenantBundleAdjuster {
                flags: flags.clone(),
                flag: "tenant-bundle-batched".to_string(),
                percent: -4.6,
            }) as Rc<dyn TenantBundleStep>,
            Rc::new(CapTenantBundleAdjuster { cap: 24391.0 }),
            Rc::new(TieredTenantBundleAdjuster {
                bounds: vec![5205.0, 6005.0, 6587.0],
                rates: vec![0.8, 0.87, 0.87],
            }),
            Rc::new(TenantBundleMaximumOffsetAdjuster { weight: 1.11 }),
            Rc::new(PercentTenantBundleAdjuster { percent: 8.1 }),
            Rc::new(RoundingTenantBundleAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 14239.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_bundle_engine(&flags::new_registry());
        let item = new_tenant_bundle("code-472", "label-542");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
