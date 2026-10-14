use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient region pin tier calculation.
pub trait LenientRegionPinTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRegionPinTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientRegionPinTierAdjuster {
    pub step: f64,
}

impl LenientRegionPinTierModifier for RoundingLenientRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientRegionPinTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientRegionPinTierAdjuster {
    pub percent: f64,
}

impl LenientRegionPinTierModifier for PercentLenientRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientRegionPinTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientRegionPinTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientRegionPinTierModifier for FlaggedLenientRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientRegionPinTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientRegionPinTierAdjuster {
    pub cap: f64,
}

impl LenientRegionPinTierModifier for CapLenientRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientRegionPinTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient region pin tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRegionPinTierEngine {
    pub steps: Vec<Rc<dyn LenientRegionPinTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRegionPinTierEngine {
    pub fn compute(&mut self, item: &LenientRegionPinTier) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[LenientRegionPinTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_region_pin_tier_engine(flags: &flags::Registry) -> LenientRegionPinTierEngine {
    LenientRegionPinTierEngine {
        steps: vec![
            Rc::new(RoundingLenientRegionPinTierAdjuster { step: 1.0 }) as Rc<dyn LenientRegionPinTierModifier>,
            Rc::new(PercentLenientRegionPinTierAdjuster { percent: -24.2 }),
            Rc::new(FlaggedLenientRegionPinTierAdjuster {
                flags: flags.clone(),
                flag: "lenient-region-pin-tier-tiered".to_string(),
                percent: 4.1,
            }),
            Rc::new(CapLenientRegionPinTierAdjuster { cap: 37490.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 69420.1,
    }
}
