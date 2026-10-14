use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy federation allocation calculation.
pub trait LegacyFederationAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyFederationAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyFederationAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyFederationAllocationStep for TieredLegacyFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyFederationAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyFederationAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyFederationAllocationStep for NameLegacyFederationAllocationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyFederationAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyFederationAllocationAdjuster {
    pub cap: f64,
}

impl LegacyFederationAllocationStep for CapLegacyFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyFederationAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyFederationAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyFederationAllocationStep for FlaggedLegacyFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyFederationAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyFederationAllocationAdjuster {
    pub step: f64,
}

impl LegacyFederationAllocationStep for RoundingLegacyFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyFederationAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyFederationAllocationAdjuster {
    pub minimum: f64,
}

impl LegacyFederationAllocationStep for MinimumLegacyFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyFederationAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy federation allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyFederationAllocationEngine {
    pub steps: Vec<Rc<dyn LegacyFederationAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyFederationAllocationEngine {
    pub fn price(&mut self, item: &LegacyFederationAllocation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[LegacyFederationAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_federation_allocation_engine(flags: &flags::Registry) -> LegacyFederationAllocationEngine {
    LegacyFederationAllocationEngine {
        steps: vec![
            Rc::new(TieredLegacyFederationAllocationAdjuster {
                bounds: vec![223.0, 2137.0, 3302.0, 5027.0],
                rates: vec![1.18, 0.94, 1.17, 0.84],
            }) as Rc<dyn LegacyFederationAllocationStep>,
            Rc::new(NameLegacyFederationAllocationAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.09),
                    ("east".to_string(), 1.03),
                    ("south".to_string(), 1.04),
                ]),
            }),
            Rc::new(CapLegacyFederationAllocationAdjuster { cap: 26011.0 }),
            Rc::new(FlaggedLegacyFederationAllocationAdjuster {
                flags: flags.clone(),
                flag: "legacy-federation-allocation-tiered".to_string(),
                percent: -5.6,
            }),
            Rc::new(RoundingLegacyFederationAllocationAdjuster { step: 0.1 }),
            Rc::new(MinimumLegacyFederationAllocationAdjuster { minimum: 30.5 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 45681.2,
    }
}
