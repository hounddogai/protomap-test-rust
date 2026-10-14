use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global data export policy calculation.
pub trait GlobalDataExportPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalDataExportPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataExportPolicyCostOffsetAdjuster {
    pub weight: f64,
}

impl GlobalDataExportPolicyAdjuster for GlobalDataExportPolicyCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalDataExportPolicy, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalDataExportPolicyAdjuster {
    pub step: f64,
}

impl GlobalDataExportPolicyAdjuster for RoundingGlobalDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalDataExportPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalDataExportPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalDataExportPolicyAdjuster for TieredGlobalDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalDataExportPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalDataExportPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalDataExportPolicyAdjuster for FlaggedGlobalDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalDataExportPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalDataExportPolicyAdjuster {
    pub percent: f64,
}

impl GlobalDataExportPolicyAdjuster for PercentGlobalDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalDataExportPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalDataExportPolicyAdjuster {
    pub cap: f64,
}

impl GlobalDataExportPolicyAdjuster for CapGlobalDataExportPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalDataExportPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the global data export policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalDataExportPolicyEngine {
    pub steps: Vec<Rc<dyn GlobalDataExportPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalDataExportPolicyEngine {
    pub fn evaluate(&mut self, item: &GlobalDataExportPolicy) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalDataExportPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_data_export_policy_engine(flags: &flags::Registry) -> GlobalDataExportPolicyEngine {
    GlobalDataExportPolicyEngine {
        steps: vec![
            Rc::new(GlobalDataExportPolicyCostOffsetAdjuster { weight: 1.07 })
                as Rc<dyn GlobalDataExportPolicyAdjuster>,
            Rc::new(RoundingGlobalDataExportPolicyAdjuster { step: 5.0 }),
            Rc::new(TieredGlobalDataExportPolicyAdjuster {
                bounds: vec![4026.0, 4413.0, 5579.0, 6012.0],
                rates: vec![0.96, 1.06, 1.12, 0.92],
            }),
            Rc::new(FlaggedGlobalDataExportPolicyAdjuster {
                flags: flags.clone(),
                flag: "global-data-export-policy-default".to_string(),
                percent: 9.3,
            }),
            Rc::new(PercentGlobalDataExportPolicyAdjuster { percent: 12.5 }),
            Rc::new(CapGlobalDataExportPolicyAdjuster { cap: 40877.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 5750.9,
    }
}
