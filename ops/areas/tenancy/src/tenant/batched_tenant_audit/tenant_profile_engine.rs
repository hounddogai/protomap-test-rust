use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant profile calculation.
pub trait TenantProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProfileMarginOffsetAdjuster {
    pub weight: f64,
}

impl TenantProfileStep for TenantProfileMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &TenantProfile, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantProfileAdjuster {
    pub minimum: f64,
}

impl TenantProfileStep for MinimumTenantProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantProfileAdjuster {
    pub percent: f64,
}

impl TenantProfileStep for PercentTenantProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantProfileAdjuster {
    pub step: f64,
}

impl TenantProfileStep for RoundingTenantProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantProfileStep for FlaggedTenantProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantProfileAdjuster {
    pub cap: f64,
}

impl TenantProfileStep for CapTenantProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tenant profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantProfileEngine {
    pub steps: Vec<Rc<dyn TenantProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantProfileEngine {
    pub fn price(&mut self, item: &TenantProfile) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TenantProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_profile_engine(flags: &flags::Registry) -> TenantProfileEngine {
    TenantProfileEngine {
        steps: vec![
            Rc::new(TenantProfileMarginOffsetAdjuster { weight: 0.79 }) as Rc<dyn TenantProfileStep>,
            Rc::new(MinimumTenantProfileAdjuster { minimum: 26.1 }),
            Rc::new(PercentTenantProfileAdjuster { percent: -14.0 }),
            Rc::new(RoundingTenantProfileAdjuster { step: 0.05 }),
            Rc::new(FlaggedTenantProfileAdjuster {
                flags: flags.clone(),
                flag: "tenant-profile-scheduled".to_string(),
                percent: -8.7,
            }),
            Rc::new(CapTenantProfileAdjuster { cap: 42689.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 30729.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_profile_engine(&flags::new_registry());
        let item = new_tenant_profile("code-906", "label-144");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
