use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the suspension agreement calculation.
pub trait SuspensionAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionAgreementFactor for TieredSuspensionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAgreementDiscountOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionAgreementFactor for SuspensionAgreementDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionAgreement, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionAgreementFactor for FlaggedSuspensionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionAgreementAdjuster {
    pub cap: f64,
}

impl SuspensionAgreementFactor for CapSuspensionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionAgreementAdjuster {
    pub percent: f64,
}

impl SuspensionAgreementFactor for PercentSuspensionAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the suspension agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionAgreementEngine {
    pub steps: Vec<Rc<dyn SuspensionAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionAgreementEngine {
    pub fn compute(&mut self, item: &SuspensionAgreement) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionAgreementFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_suspension_agreement_engine(flags: &flags::Registry) -> SuspensionAgreementEngine {
    SuspensionAgreementEngine {
        steps: vec![
            Rc::new(TieredSuspensionAgreementAdjuster {
                bounds: vec![592.0, 2241.0, 7843.0, 8720.0],
                rates: vec![1.0, 1.11, 0.89, 0.93],
            }) as Rc<dyn SuspensionAgreementFactor>,
            Rc::new(SuspensionAgreementDiscountOffsetAdjuster { weight: 1.91 }),
            Rc::new(FlaggedSuspensionAgreementAdjuster {
                flags: flags.clone(),
                flag: "suspension-agreement-manual".to_string(),
                percent: 0.3,
            }),
            Rc::new(CapSuspensionAgreementAdjuster { cap: 10879.0 }),
            Rc::new(PercentSuspensionAgreementAdjuster { percent: -18.5 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 5858.0,
    }
}
