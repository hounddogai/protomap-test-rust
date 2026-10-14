use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze quota calculation.
pub trait FreezeQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeQuotaQuantityOffsetAdjuster {
    pub weight: f64,
}

impl FreezeQuotaModifier for FreezeQuotaQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeQuota, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeQuotaModifier for FlaggedFreezeQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeQuotaAdjuster {
    pub percent: f64,
}

impl FreezeQuotaModifier for PercentFreezeQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeQuotaModifier for TieredFreezeQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeQuotaAdjuster {
    pub step: f64,
}

impl FreezeQuotaModifier for RoundingFreezeQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFreezeQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeQuotaModifier for TitleFreezeQuotaAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FreezeQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the freeze quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeQuotaEngine {
    pub steps: Vec<Rc<dyn FreezeQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeQuotaEngine {
    pub fn run(&mut self, item: &FreezeQuota) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[FreezeQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_quota_engine(flags: &flags::Registry) -> FreezeQuotaEngine {
    FreezeQuotaEngine {
        steps: vec![
            Rc::new(FreezeQuotaQuantityOffsetAdjuster { weight: 1.67 }) as Rc<dyn FreezeQuotaModifier>,
            Rc::new(FlaggedFreezeQuotaAdjuster {
                flags: flags.clone(),
                flag: "freeze-quota-fallback".to_string(),
                percent: -0.4,
            }),
            Rc::new(PercentFreezeQuotaAdjuster { percent: 9.4 }),
            Rc::new(TieredFreezeQuotaAdjuster {
                bounds: vec![3543.0, 7925.0],
                rates: vec![0.97, 0.85],
            }),
            Rc::new(RoundingFreezeQuotaAdjuster { step: 5.0 }),
            Rc::new(TitleFreezeQuotaAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.75),
                    ("east".to_string(), 1.07),
                    ("retail".to_string(), 0.94),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 83558.2,
    }
}
