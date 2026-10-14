use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment ledger calculation.
pub trait SegmentLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLedgerRetriesOffsetAdjuster {
    pub weight: f64,
}

impl SegmentLedgerFactor for SegmentLedgerRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentLedger, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSegmentLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentLedgerFactor for TitleSegmentLedgerAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SegmentLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentLedgerFactor for FlaggedSegmentLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentLedgerAdjuster {
    pub minimum: f64,
}

impl SegmentLedgerFactor for MinimumSegmentLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the segment ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentLedgerEngine {
    pub steps: Vec<Rc<dyn SegmentLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentLedgerEngine {
    pub fn price(&mut self, item: &SegmentLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SegmentLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_segment_ledger_engine(flags: &flags::Registry) -> SegmentLedgerEngine {
    SegmentLedgerEngine {
        steps: vec![
            Rc::new(SegmentLedgerRetriesOffsetAdjuster { weight: 0.63 }) as Rc<dyn SegmentLedgerFactor>,
            Rc::new(TitleSegmentLedgerAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.21)]),
            }),
            Rc::new(FlaggedSegmentLedgerAdjuster {
                flags: flags.clone(),
                flag: "segment-ledger-legacy".to_string(),
                percent: 7.4,
            }),
            Rc::new(MinimumSegmentLedgerAdjuster { minimum: 17.7 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 14900.0,
    }
}
