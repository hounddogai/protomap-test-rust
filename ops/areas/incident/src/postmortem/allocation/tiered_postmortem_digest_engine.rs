use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered postmortem digest calculation.
pub trait TieredPostmortemDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPostmortemDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredPostmortemDigestAdjuster {
    pub step: f64,
}

impl TieredPostmortemDigestStep for RoundingTieredPostmortemDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredPostmortemDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredPostmortemDigestAdjuster {
    pub percent: f64,
}

impl TieredPostmortemDigestStep for PercentTieredPostmortemDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredPostmortemDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredPostmortemDigestAdjuster {
    pub minimum: f64,
}

impl TieredPostmortemDigestStep for MinimumTieredPostmortemDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredPostmortemDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemDigestRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl TieredPostmortemDigestStep for TieredPostmortemDigestRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &TieredPostmortemDigest, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

/// Runs the tiered postmortem digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPostmortemDigestEngine {
    pub steps: Vec<Rc<dyn TieredPostmortemDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPostmortemDigestEngine {
    pub fn compute(&mut self, item: &TieredPostmortemDigest) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[TieredPostmortemDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_postmortem_digest_engine() -> TieredPostmortemDigestEngine {
    TieredPostmortemDigestEngine {
        steps: vec![
            Rc::new(RoundingTieredPostmortemDigestAdjuster { step: 0.5 }) as Rc<dyn TieredPostmortemDigestStep>,
            Rc::new(PercentTieredPostmortemDigestAdjuster { percent: -10.0 }),
            Rc::new(MinimumTieredPostmortemDigestAdjuster { minimum: 46.9 }),
            Rc::new(TieredPostmortemDigestRotationDaysOffsetAdjuster { weight: 1.1 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 45153.6,
    }
}
