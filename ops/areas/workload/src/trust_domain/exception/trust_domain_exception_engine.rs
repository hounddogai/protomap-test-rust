use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the trust domain exception calculation.
pub trait TrustDomainExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainExceptionAdjuster {
    pub minimum: f64,
}

impl TrustDomainExceptionStep for MinimumTrustDomainExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainExceptionRateOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainExceptionStep for TrustDomainExceptionRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainException, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainExceptionAdjuster {
    pub percent: f64,
}

impl TrustDomainExceptionStep for PercentTrustDomainExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainExceptionStep for TieredTrustDomainExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainExceptionStep for FlaggedTrustDomainExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the trust domain exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainExceptionEngine {
    pub steps: Vec<Rc<dyn TrustDomainExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainExceptionEngine {
    pub fn price(&mut self, item: &TrustDomainException) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainExceptionStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_trust_domain_exception_engine(flags: &flags::Registry) -> TrustDomainExceptionEngine {
    TrustDomainExceptionEngine {
        steps: vec![
            Rc::new(MinimumTrustDomainExceptionAdjuster { minimum: 41.9 }) as Rc<dyn TrustDomainExceptionStep>,
            Rc::new(TrustDomainExceptionRateOffsetAdjuster { weight: 0.87 }),
            Rc::new(PercentTrustDomainExceptionAdjuster { percent: -9.6 }),
            Rc::new(TieredTrustDomainExceptionAdjuster {
                bounds: vec![3660.0, 5602.0, 7873.0],
                rates: vec![0.92, 1.09, 0.99],
            }),
            Rc::new(FlaggedTrustDomainExceptionAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-exception-tiered".to_string(),
                percent: 1.6,
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 49445.1,
    }
}
