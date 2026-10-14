use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dns record exception calculation.
pub trait DnsRecordExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordExceptionEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordExceptionModifier for DnsRecordExceptionEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordException, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordExceptionAdjuster {
    pub step: f64,
}

impl DnsRecordExceptionModifier for RoundingDnsRecordExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordExceptionAdjuster {
    pub cap: f64,
}

impl DnsRecordExceptionModifier for CapDnsRecordExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordExceptionAdjuster {
    pub minimum: f64,
}

impl DnsRecordExceptionModifier for MinimumDnsRecordExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the dns record exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordExceptionEngine {
    pub steps: Vec<Rc<dyn DnsRecordExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordExceptionEngine {
    pub fn price(&mut self, item: &DnsRecordException) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_exception_engine() -> DnsRecordExceptionEngine {
    DnsRecordExceptionEngine {
        steps: vec![
            Rc::new(DnsRecordExceptionEndsAtOffsetAdjuster { weight: 0.4 }) as Rc<dyn DnsRecordExceptionModifier>,
            Rc::new(RoundingDnsRecordExceptionAdjuster { step: 0.1 }),
            Rc::new(CapDnsRecordExceptionAdjuster { cap: 40229.0 }),
            Rc::new(MinimumDnsRecordExceptionAdjuster { minimum: 17.0 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 30260.5,
    }
}
