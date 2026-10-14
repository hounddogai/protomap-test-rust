use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary dns record summary calculation.
pub trait PrimaryDnsRecordSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryDnsRecordSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryDnsRecordSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryDnsRecordSummaryFactor for FlaggedPrimaryDnsRecordSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryDnsRecordSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryDnsRecordSummaryAdjuster {
    pub minimum: f64,
}

impl PrimaryDnsRecordSummaryFactor for MinimumPrimaryDnsRecordSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryDnsRecordSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryDnsRecordSummaryAdjuster {
    pub percent: f64,
}

impl PrimaryDnsRecordSummaryFactor for PercentPrimaryDnsRecordSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryDnsRecordSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryDnsRecordSummaryAdjuster {
    pub cap: f64,
}

impl PrimaryDnsRecordSummaryFactor for CapPrimaryDnsRecordSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryDnsRecordSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary dns record summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryDnsRecordSummaryEngine {
    pub steps: Vec<Rc<dyn PrimaryDnsRecordSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryDnsRecordSummaryEngine {
    pub fn compute(&mut self, item: &PrimaryDnsRecordSummary) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryDnsRecordSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_dns_record_summary_engine(flags: &flags::Registry) -> PrimaryDnsRecordSummaryEngine {
    PrimaryDnsRecordSummaryEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryDnsRecordSummaryAdjuster {
                flags: flags.clone(),
                flag: "primary-dns-record-summary-tiered".to_string(),
                percent: -3.8,
            }) as Rc<dyn PrimaryDnsRecordSummaryFactor>,
            Rc::new(MinimumPrimaryDnsRecordSummaryAdjuster { minimum: 49.5 }),
            Rc::new(PercentPrimaryDnsRecordSummaryAdjuster { percent: 23.7 }),
            Rc::new(CapPrimaryDnsRecordSummaryAdjuster { cap: 38337.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 67858.7,
    }
}
