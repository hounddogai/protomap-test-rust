use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional runbook mapping calculation.
pub trait RegionalRunbookMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRunbookMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalRunbookMappingAdjuster {
    pub minimum: f64,
}

impl RegionalRunbookMappingStep for MinimumRegionalRunbookMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalRunbookMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookMappingBudgetOffsetAdjuster {
    pub weight: f64,
}

impl RegionalRunbookMappingStep for RegionalRunbookMappingBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalRunbookMapping, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalRunbookMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalRunbookMappingStep for FlaggedRegionalRunbookMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalRunbookMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalRunbookMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalRunbookMappingStep for TieredRegionalRunbookMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalRunbookMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional runbook mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRunbookMappingEngine {
    pub steps: Vec<Rc<dyn RegionalRunbookMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRunbookMappingEngine {
    pub fn evaluate(&mut self, item: &RegionalRunbookMapping) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalRunbookMappingStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRunbookMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_runbook_mapping_engine(flags: &flags::Registry) -> RegionalRunbookMappingEngine {
    RegionalRunbookMappingEngine {
        steps: vec![
            Rc::new(MinimumRegionalRunbookMappingAdjuster { minimum: 2.0 }) as Rc<dyn RegionalRunbookMappingStep>,
            Rc::new(RegionalRunbookMappingBudgetOffsetAdjuster { weight: 1.54 }),
            Rc::new(FlaggedRegionalRunbookMappingAdjuster {
                flags: flags.clone(),
                flag: "regional-runbook-mapping-scheduled".to_string(),
                percent: -2.1,
            }),
            Rc::new(TieredRegionalRunbookMappingAdjuster {
                bounds: vec![1900.0, 2738.0],
                rates: vec![1.15, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 47100.2,
    }
}
