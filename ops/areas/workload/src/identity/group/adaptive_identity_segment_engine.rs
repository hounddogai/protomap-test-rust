use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive identity segment calculation.
pub trait AdaptiveIdentitySegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveIdentitySegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentitySegmentBudgetOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveIdentitySegmentAdjuster for AdaptiveIdentitySegmentBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveIdentitySegment, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveIdentitySegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveIdentitySegmentAdjuster for FlaggedAdaptiveIdentitySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveIdentitySegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveIdentitySegmentAdjuster {
    pub percent: f64,
}

impl AdaptiveIdentitySegmentAdjuster for PercentAdaptiveIdentitySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveIdentitySegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveIdentitySegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveIdentitySegmentAdjuster for TieredAdaptiveIdentitySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveIdentitySegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive identity segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveIdentitySegmentEngine {
    pub steps: Vec<Rc<dyn AdaptiveIdentitySegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveIdentitySegmentEngine {
    pub fn run(&mut self, item: &AdaptiveIdentitySegment) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveIdentitySegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_identity_segment_engine(flags: &flags::Registry) -> AdaptiveIdentitySegmentEngine {
    AdaptiveIdentitySegmentEngine {
        steps: vec![
            Rc::new(AdaptiveIdentitySegmentBudgetOffsetAdjuster { weight: 0.64 })
                as Rc<dyn AdaptiveIdentitySegmentAdjuster>,
            Rc::new(FlaggedAdaptiveIdentitySegmentAdjuster {
                flags: flags.clone(),
                flag: "adaptive-identity-segment-tiered".to_string(),
                percent: 7.1,
            }),
            Rc::new(PercentAdaptiveIdentitySegmentAdjuster { percent: -17.8 }),
            Rc::new(TieredAdaptiveIdentitySegmentAdjuster {
                bounds: vec![365.0, 3579.0],
                rates: vec![1.0, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 95101.6,
    }
}
