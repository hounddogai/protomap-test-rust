use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the suspension exception calculation.
pub trait SuspensionExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionExceptionStep for FlaggedSuspensionExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionExceptionStep for TieredSuspensionExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSuspensionExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionExceptionStep for NameSuspensionExceptionAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SuspensionException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionExceptionAdjuster {
    pub step: f64,
}

impl SuspensionExceptionStep for RoundingSuspensionExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionExceptionQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionExceptionStep for SuspensionExceptionQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionException, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the suspension exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionExceptionEngine {
    pub steps: Vec<Rc<dyn SuspensionExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionExceptionEngine {
    pub fn compute(&mut self, item: &SuspensionException) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_suspension_exception_engine(flags: &flags::Registry) -> SuspensionExceptionEngine {
    SuspensionExceptionEngine {
        steps: vec![
            Rc::new(FlaggedSuspensionExceptionAdjuster {
                flags: flags.clone(),
                flag: "suspension-exception-global".to_string(),
                percent: 4.9,
            }) as Rc<dyn SuspensionExceptionStep>,
            Rc::new(TieredSuspensionExceptionAdjuster {
                bounds: vec![1584.0, 5172.0, 7621.0, 9015.0],
                rates: vec![1.2, 1.11, 0.94, 0.94],
            }),
            Rc::new(NameSuspensionExceptionAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.06)]),
            }),
            Rc::new(RoundingSuspensionExceptionAdjuster { step: 0.05 }),
            Rc::new(SuspensionExceptionQuotaUnitsOffsetAdjuster { weight: 1.45 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 22702.9,
    }
}
