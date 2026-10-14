use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual dns record template calculation.
pub trait ManualDnsRecordTemplateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDnsRecordTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualDnsRecordTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualDnsRecordTemplateStep for FlaggedManualDnsRecordTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualDnsRecordTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualDnsRecordTemplateAdjuster {
    pub cap: f64,
}

impl ManualDnsRecordTemplateStep for CapManualDnsRecordTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualDnsRecordTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualDnsRecordTemplateAdjuster {
    pub step: f64,
}

impl ManualDnsRecordTemplateStep for RoundingManualDnsRecordTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualDnsRecordTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualDnsRecordTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualDnsRecordTemplateStep for TieredManualDnsRecordTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualDnsRecordTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordTemplateWeightOffsetAdjuster {
    pub weight: f64,
}

impl ManualDnsRecordTemplateStep for ManualDnsRecordTemplateWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &ManualDnsRecordTemplate, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualDnsRecordTemplateAdjuster {
    pub minimum: f64,
}

impl ManualDnsRecordTemplateStep for MinimumManualDnsRecordTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualDnsRecordTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual dns record template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDnsRecordTemplateEngine {
    pub steps: Vec<Rc<dyn ManualDnsRecordTemplateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDnsRecordTemplateEngine {
    pub fn evaluate(&mut self, item: &ManualDnsRecordTemplate) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualDnsRecordTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_dns_record_template_engine(flags: &flags::Registry) -> ManualDnsRecordTemplateEngine {
    ManualDnsRecordTemplateEngine {
        steps: vec![
            Rc::new(FlaggedManualDnsRecordTemplateAdjuster {
                flags: flags.clone(),
                flag: "manual-dns-record-template-lenient".to_string(),
                percent: -8.8,
            }) as Rc<dyn ManualDnsRecordTemplateStep>,
            Rc::new(CapManualDnsRecordTemplateAdjuster { cap: 15555.0 }),
            Rc::new(RoundingManualDnsRecordTemplateAdjuster { step: 0.5 }),
            Rc::new(TieredManualDnsRecordTemplateAdjuster {
                bounds: vec![2234.0, 2370.0, 3676.0],
                rates: vec![1.0, 0.86, 1.04],
            }),
            Rc::new(ManualDnsRecordTemplateWeightOffsetAdjuster { weight: 0.86 }),
            Rc::new(MinimumManualDnsRecordTemplateAdjuster { minimum: 24.1 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 6114.0,
    }
}
