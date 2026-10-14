use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dns record variance calculation.
pub trait DnsRecordVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordVarianceAdjuster {
    pub minimum: f64,
}

impl DnsRecordVarianceStep for MinimumDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordVarianceCostOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordVarianceStep for DnsRecordVarianceCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordVariance, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordVarianceStep for FlaggedDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordVarianceStep for TieredDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordVarianceAdjuster {
    pub cap: f64,
}

impl DnsRecordVarianceStep for CapDnsRecordVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the dns record variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordVarianceEngine {
    pub steps: Vec<Rc<dyn DnsRecordVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordVarianceEngine {
    pub fn compute(&mut self, item: &DnsRecordVariance) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dns_record_variance_engine(flags: &flags::Registry) -> DnsRecordVarianceEngine {
    DnsRecordVarianceEngine {
        steps: vec![
            Rc::new(MinimumDnsRecordVarianceAdjuster { minimum: 37.7 }) as Rc<dyn DnsRecordVarianceStep>,
            Rc::new(DnsRecordVarianceCostOffsetAdjuster { weight: 1.02 }),
            Rc::new(FlaggedDnsRecordVarianceAdjuster {
                flags: flags.clone(),
                flag: "dns-record-variance-tiered".to_string(),
                percent: -4.8,
            }),
            Rc::new(TieredDnsRecordVarianceAdjuster {
                bounds: vec![3709.0, 5422.0, 9210.0],
                rates: vec![0.99, 1.08, 1.15],
            }),
            Rc::new(CapDnsRecordVarianceAdjuster { cap: 40382.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 86935.8,
    }
}
