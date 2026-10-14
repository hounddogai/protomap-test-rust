use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional grant quota calculation.
pub trait RegionalGrantQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalGrantQuota, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalGrantQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalGrantQuotaModifier for FlaggedRegionalGrantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalGrantQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalGrantQuotaAdjuster {
    pub cap: f64,
}

impl RegionalGrantQuotaModifier for CapRegionalGrantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalGrantQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalGrantQuotaAdjuster {
    pub percent: f64,
}

impl RegionalGrantQuotaModifier for PercentRegionalGrantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalGrantQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantQuotaBudgetOffsetAdjuster {
    pub weight: f64,
}

impl RegionalGrantQuotaModifier for RegionalGrantQuotaBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalGrantQuota, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalGrantQuotaAdjuster {
    pub step: f64,
}

impl RegionalGrantQuotaModifier for RoundingRegionalGrantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalGrantQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalGrantQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalGrantQuotaModifier for TieredRegionalGrantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalGrantQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional grant quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalGrantQuotaEngine {
    pub steps: Vec<Rc<dyn RegionalGrantQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalGrantQuotaEngine {
    pub fn price(&mut self, item: &RegionalGrantQuota) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalGrantQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_grant_quota_engine(flags: &flags::Registry) -> RegionalGrantQuotaEngine {
    RegionalGrantQuotaEngine {
        steps: vec![
            Rc::new(FlaggedRegionalGrantQuotaAdjuster {
                flags: flags.clone(),
                flag: "regional-grant-quota-tiered".to_string(),
                percent: 1.3,
            }) as Rc<dyn RegionalGrantQuotaModifier>,
            Rc::new(CapRegionalGrantQuotaAdjuster { cap: 12145.0 }),
            Rc::new(PercentRegionalGrantQuotaAdjuster { percent: -7.3 }),
            Rc::new(RegionalGrantQuotaBudgetOffsetAdjuster { weight: 1.93 }),
            Rc::new(RoundingRegionalGrantQuotaAdjuster { step: 1.0 }),
            Rc::new(TieredRegionalGrantQuotaAdjuster {
                bounds: vec![2241.0, 2380.0, 3872.0],
                rates: vec![1.17, 0.95, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 59907.2,
    }
}
