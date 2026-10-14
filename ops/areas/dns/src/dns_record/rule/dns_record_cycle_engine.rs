use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the dns record cycle calculation.
pub trait DnsRecordCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordCycleAdjuster {
    pub cap: f64,
}

impl DnsRecordCycleFactor for CapDnsRecordCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordCycleAdjuster {
    pub minimum: f64,
}

impl DnsRecordCycleFactor for MinimumDnsRecordCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDnsRecordCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordCycleFactor for TitleDnsRecordCycleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DnsRecordCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordCycleFactor for TieredDnsRecordCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the dns record cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordCycleEngine {
    pub steps: Vec<Rc<dyn DnsRecordCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordCycleEngine {
    pub fn price(&mut self, item: &DnsRecordCycle) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordCycleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_cycle_engine() -> DnsRecordCycleEngine {
    DnsRecordCycleEngine {
        steps: vec![
            Rc::new(CapDnsRecordCycleAdjuster { cap: 24586.0 }) as Rc<dyn DnsRecordCycleFactor>,
            Rc::new(MinimumDnsRecordCycleAdjuster { minimum: 26.5 }),
            Rc::new(TitleDnsRecordCycleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.13), ("south".to_string(), 1.33)]),
            }),
            Rc::new(TieredDnsRecordCycleAdjuster {
                bounds: vec![5258.0, 5443.0, 7278.0],
                rates: vec![1.11, 1.18, 0.84],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 25588.9,
    }
}
