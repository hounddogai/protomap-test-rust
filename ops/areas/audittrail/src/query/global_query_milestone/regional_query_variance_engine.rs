use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional query variance calculation.
pub trait RegionalQueryVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalQueryVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalQueryVarianceAdjuster {
    pub percent: f64,
}

impl RegionalQueryVarianceModifier for PercentRegionalQueryVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalQueryVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalQueryVarianceAdjuster {
    pub cap: f64,
}

impl RegionalQueryVarianceModifier for CapRegionalQueryVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalQueryVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryVarianceRevisionOffsetAdjuster {
    pub weight: f64,
}

impl RegionalQueryVarianceModifier for RegionalQueryVarianceRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalQueryVariance, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalQueryVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalQueryVarianceModifier for FlaggedRegionalQueryVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalQueryVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalQueryVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalQueryVarianceModifier for TieredRegionalQueryVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalQueryVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional query variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalQueryVarianceEngine {
    pub steps: Vec<Rc<dyn RegionalQueryVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalQueryVarianceEngine {
    pub fn run(&mut self, item: &RegionalQueryVariance) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalQueryVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_query_variance_engine(flags: &flags::Registry) -> RegionalQueryVarianceEngine {
    RegionalQueryVarianceEngine {
        steps: vec![
            Rc::new(PercentRegionalQueryVarianceAdjuster { percent: 16.9 }) as Rc<dyn RegionalQueryVarianceModifier>,
            Rc::new(CapRegionalQueryVarianceAdjuster { cap: 29325.0 }),
            Rc::new(RegionalQueryVarianceRevisionOffsetAdjuster { weight: 0.89 }),
            Rc::new(FlaggedRegionalQueryVarianceAdjuster {
                flags: flags.clone(),
                flag: "regional-query-variance-tiered".to_string(),
                percent: -9.4,
            }),
            Rc::new(TieredRegionalQueryVarianceAdjuster {
                bounds: vec![1966.0, 4903.0],
                rates: vec![0.91, 1.14],
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 18286.7,
    }
}
