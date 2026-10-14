use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global plan quota calculation.
pub trait GlobalPlanQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPlanQuota, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalPlanQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalPlanQuotaAdjuster for FlaggedGlobalPlanQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalPlanQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceGlobalPlanQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalPlanQuotaAdjuster for SourceGlobalPlanQuotaAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &GlobalPlanQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalPlanQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalPlanQuotaAdjuster for TieredGlobalPlanQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalPlanQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPlanQuotaAdjuster {
    pub cap: f64,
}

impl GlobalPlanQuotaAdjuster for CapGlobalPlanQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPlanQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the global plan quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPlanQuotaEngine {
    pub steps: Vec<Rc<dyn GlobalPlanQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPlanQuotaEngine {
    pub fn compute(&mut self, item: &GlobalPlanQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GlobalPlanQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_plan_quota_engine(flags: &flags::Registry) -> GlobalPlanQuotaEngine {
    GlobalPlanQuotaEngine {
        steps: vec![
            Rc::new(FlaggedGlobalPlanQuotaAdjuster {
                flags: flags.clone(),
                flag: "global-plan-quota-manual".to_string(),
                percent: 0.9,
            }) as Rc<dyn GlobalPlanQuotaAdjuster>,
            Rc::new(SourceGlobalPlanQuotaAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.06)]),
            }),
            Rc::new(TieredGlobalPlanQuotaAdjuster {
                bounds: vec![799.0, 3540.0, 4195.0, 5855.0],
                rates: vec![1.11, 0.89, 0.92, 1.06],
            }),
            Rc::new(CapGlobalPlanQuotaAdjuster { cap: 40833.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 18197.1,
    }
}
