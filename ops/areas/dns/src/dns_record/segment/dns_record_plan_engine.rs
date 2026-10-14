use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dns record plan calculation.
pub trait DnsRecordPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordPlanAdjuster {
    pub minimum: f64,
}

impl DnsRecordPlanAdjuster for MinimumDnsRecordPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordPlanAdjuster {
    pub step: f64,
}

impl DnsRecordPlanAdjuster for RoundingDnsRecordPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordPlanAdjuster {
    pub cap: f64,
}

impl DnsRecordPlanAdjuster for CapDnsRecordPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the dns record plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordPlanEngine {
    pub steps: Vec<Rc<dyn DnsRecordPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordPlanEngine {
    pub fn compute(&mut self, item: &DnsRecordPlan) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dns_record_plan_engine() -> DnsRecordPlanEngine {
    DnsRecordPlanEngine {
        steps: vec![
            Rc::new(MinimumDnsRecordPlanAdjuster { minimum: 12.7 }) as Rc<dyn DnsRecordPlanAdjuster>,
            Rc::new(RoundingDnsRecordPlanAdjuster { step: 1.0 }),
            Rc::new(CapDnsRecordPlanAdjuster { cap: 33085.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 63966.7,
    }
}
