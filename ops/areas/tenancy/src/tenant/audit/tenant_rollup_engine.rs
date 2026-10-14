use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant rollup calculation.
pub trait TenantRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantRollupAdjuster {
    pub percent: f64,
}

impl TenantRollupAdjuster for PercentTenantRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantRollupAdjuster {
    pub cap: f64,
}

impl TenantRollupAdjuster for CapTenantRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantRollupAdjuster for FlaggedTenantRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantRollupAdjuster for TieredTenantRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tenant rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantRollupEngine {
    pub steps: Vec<Rc<dyn TenantRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantRollupEngine {
    pub fn run(&mut self, item: &TenantRollup) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TenantRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tenant_rollup_engine(flags: &flags::Registry) -> TenantRollupEngine {
    TenantRollupEngine {
        steps: vec![
            Rc::new(PercentTenantRollupAdjuster { percent: 0.6 }) as Rc<dyn TenantRollupAdjuster>,
            Rc::new(CapTenantRollupAdjuster { cap: 14858.0 }),
            Rc::new(FlaggedTenantRollupAdjuster {
                flags: flags.clone(),
                flag: "tenant-rollup-capped".to_string(),
                percent: -4.2,
            }),
            Rc::new(TieredTenantRollupAdjuster {
                bounds: vec![3005.0, 7703.0],
                rates: vec![0.81, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 37142.0,
    }
}
