use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dns record assignment calculation.
pub trait DnsRecordAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordAssignmentAdjuster {
    pub step: f64,
}

impl DnsRecordAssignmentModifier for RoundingDnsRecordAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordAssignmentAdjuster {
    pub cap: f64,
}

impl DnsRecordAssignmentModifier for CapDnsRecordAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordAssignmentModifier for FlaggedDnsRecordAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordAssignmentAdjuster {
    pub minimum: f64,
}

impl DnsRecordAssignmentModifier for MinimumDnsRecordAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordAssignmentAdjuster {
    pub percent: f64,
}

impl DnsRecordAssignmentModifier for PercentDnsRecordAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dns record assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordAssignmentEngine {
    pub steps: Vec<Rc<dyn DnsRecordAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordAssignmentEngine {
    pub fn price(&mut self, item: &DnsRecordAssignment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_assignment_engine(flags: &flags::Registry) -> DnsRecordAssignmentEngine {
    DnsRecordAssignmentEngine {
        steps: vec![
            Rc::new(RoundingDnsRecordAssignmentAdjuster { step: 5.0 }) as Rc<dyn DnsRecordAssignmentModifier>,
            Rc::new(CapDnsRecordAssignmentAdjuster { cap: 27013.0 }),
            Rc::new(FlaggedDnsRecordAssignmentAdjuster {
                flags: flags.clone(),
                flag: "dns-record-assignment-default".to_string(),
                percent: -4.2,
            }),
            Rc::new(MinimumDnsRecordAssignmentAdjuster { minimum: 43.1 }),
            Rc::new(PercentDnsRecordAssignmentAdjuster { percent: -20.2 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 34623.9,
    }
}
