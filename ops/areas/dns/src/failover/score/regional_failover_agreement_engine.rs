use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional failover agreement calculation.
pub trait RegionalFailoverAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalFailoverAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRegionalFailoverAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalFailoverAgreementModifier for LocaleRegionalFailoverAgreementAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RegionalFailoverAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalFailoverAgreementAdjuster {
    pub cap: f64,
}

impl RegionalFailoverAgreementModifier for CapRegionalFailoverAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalFailoverAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverAgreementRateOffsetAdjuster {
    pub weight: f64,
}

impl RegionalFailoverAgreementModifier for RegionalFailoverAgreementRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalFailoverAgreement, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalFailoverAgreementAdjuster {
    pub minimum: f64,
}

impl RegionalFailoverAgreementModifier for MinimumRegionalFailoverAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalFailoverAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalFailoverAgreementAdjuster {
    pub step: f64,
}

impl RegionalFailoverAgreementModifier for RoundingRegionalFailoverAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalFailoverAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalFailoverAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalFailoverAgreementModifier for TieredRegionalFailoverAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalFailoverAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional failover agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalFailoverAgreementEngine {
    pub steps: Vec<Rc<dyn RegionalFailoverAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalFailoverAgreementEngine {
    pub fn run(&mut self, item: &RegionalFailoverAgreement) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalFailoverAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_failover_agreement_engine() -> RegionalFailoverAgreementEngine {
    RegionalFailoverAgreementEngine {
        steps: vec![
            Rc::new(LocaleRegionalFailoverAgreementAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.25), ("east".to_string(), 0.78)]),
            }) as Rc<dyn RegionalFailoverAgreementModifier>,
            Rc::new(CapRegionalFailoverAgreementAdjuster { cap: 4586.0 }),
            Rc::new(RegionalFailoverAgreementRateOffsetAdjuster { weight: 0.09 }),
            Rc::new(MinimumRegionalFailoverAgreementAdjuster { minimum: 16.8 }),
            Rc::new(RoundingRegionalFailoverAgreementAdjuster { step: 0.1 }),
            Rc::new(TieredRegionalFailoverAgreementAdjuster {
                bounds: vec![4835.0, 4839.0, 8344.0],
                rates: vec![1.0, 0.94, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 96052.4,
    }
}
