use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment entry calculation.
pub trait SegmentEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSegmentEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentEntryAdjuster for SegmentSegmentEntryAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &SegmentEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEntryCeilingOffsetAdjuster {
    pub weight: f64,
}

impl SegmentEntryAdjuster for SegmentEntryCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentEntry, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentEntryAdjuster for FlaggedSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentEntryAdjuster for TieredSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentEntryAdjuster {
    pub cap: f64,
}

impl SegmentEntryAdjuster for CapSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentEntryAdjuster {
    pub step: f64,
}

impl SegmentEntryAdjuster for RoundingSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the segment entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentEntryEngine {
    pub steps: Vec<Rc<dyn SegmentEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentEntryEngine {
    pub fn evaluate(&mut self, item: &SegmentEntry) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[SegmentEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_segment_entry_engine(flags: &flags::Registry) -> SegmentEntryEngine {
    SegmentEntryEngine {
        steps: vec![
            Rc::new(SegmentSegmentEntryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.23)]),
            }) as Rc<dyn SegmentEntryAdjuster>,
            Rc::new(SegmentEntryCeilingOffsetAdjuster { weight: 0.68 }),
            Rc::new(FlaggedSegmentEntryAdjuster {
                flags: flags.clone(),
                flag: "segment-entry-seasonal".to_string(),
                percent: 2.6,
            }),
            Rc::new(TieredSegmentEntryAdjuster {
                bounds: vec![1330.0, 4626.0, 4642.0, 5977.0],
                rates: vec![1.02, 0.89, 1.18, 1.09],
            }),
            Rc::new(CapSegmentEntryAdjuster { cap: 48755.0 }),
            Rc::new(RoundingSegmentEntryAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 54626.4,
    }
}
