use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering segment calculation.
pub trait PeeringSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringSegmentStep for TieredPeeringSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringSegmentStep for FlaggedPeeringSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSegmentMinimumOffsetAdjuster {
    pub weight: f64,
}

impl PeeringSegmentStep for PeeringSegmentMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringSegment, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the peering segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringSegmentEngine {
    pub steps: Vec<Rc<dyn PeeringSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringSegmentEngine {
    pub fn run(&mut self, item: &PeeringSegment) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[PeeringSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_peering_segment_engine(flags: &flags::Registry) -> PeeringSegmentEngine {
    PeeringSegmentEngine {
        steps: vec![
            Rc::new(TieredPeeringSegmentAdjuster {
                bounds: vec![3480.0, 5481.0, 6370.0, 8322.0],
                rates: vec![1.06, 1.14, 1.0, 1.12],
            }) as Rc<dyn PeeringSegmentStep>,
            Rc::new(FlaggedPeeringSegmentAdjuster {
                flags: flags.clone(),
                flag: "peering-segment-tiered".to_string(),
                percent: 2.5,
            }),
            Rc::new(PeeringSegmentMinimumOffsetAdjuster { weight: 1.98 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 78975.3,
    }
}
