use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional dns record snapshot calculation.
pub trait RegionalDnsRecordSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDnsRecordSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalDnsRecordSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalDnsRecordSnapshotModifier for FlaggedRegionalDnsRecordSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalDnsRecordSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalDnsRecordSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalDnsRecordSnapshotModifier for TieredRegionalDnsRecordSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalDnsRecordSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalDnsRecordSnapshotAdjuster {
    pub percent: f64,
}

impl RegionalDnsRecordSnapshotModifier for PercentRegionalDnsRecordSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalDnsRecordSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalDnsRecordSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalDnsRecordSnapshotModifier for NameRegionalDnsRecordSnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalDnsRecordSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalDnsRecordSnapshotAdjuster {
    pub minimum: f64,
}

impl RegionalDnsRecordSnapshotModifier for MinimumRegionalDnsRecordSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalDnsRecordSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalDnsRecordSnapshotAdjuster {
    pub step: f64,
}

impl RegionalDnsRecordSnapshotModifier for RoundingRegionalDnsRecordSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalDnsRecordSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional dns record snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDnsRecordSnapshotEngine {
    pub steps: Vec<Rc<dyn RegionalDnsRecordSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDnsRecordSnapshotEngine {
    pub fn run(&mut self, item: &RegionalDnsRecordSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionalDnsRecordSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_dns_record_snapshot_engine(flags: &flags::Registry) -> RegionalDnsRecordSnapshotEngine {
    RegionalDnsRecordSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedRegionalDnsRecordSnapshotAdjuster {
                flags: flags.clone(),
                flag: "regional-dns-record-snapshot-default".to_string(),
                percent: 1.2,
            }) as Rc<dyn RegionalDnsRecordSnapshotModifier>,
            Rc::new(TieredRegionalDnsRecordSnapshotAdjuster {
                bounds: vec![1370.0, 2404.0, 3141.0],
                rates: vec![1.07, 1.12, 0.88],
            }),
            Rc::new(PercentRegionalDnsRecordSnapshotAdjuster { percent: -5.5 }),
            Rc::new(NameRegionalDnsRecordSnapshotAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.84), ("south".to_string(), 1.37)]),
            }),
            Rc::new(MinimumRegionalDnsRecordSnapshotAdjuster { minimum: 2.8 }),
            Rc::new(RoundingRegionalDnsRecordSnapshotAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 19844.3,
    }
}
