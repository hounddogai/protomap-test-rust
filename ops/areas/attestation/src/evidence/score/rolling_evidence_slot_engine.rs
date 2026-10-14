use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling evidence slot calculation.
pub trait RollingEvidenceSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingEvidenceSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingEvidenceSlotAdjuster {
    pub cap: f64,
}

impl RollingEvidenceSlotModifier for CapRollingEvidenceSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingEvidenceSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingEvidenceSlotAdjuster {
    pub step: f64,
}

impl RollingEvidenceSlotModifier for RoundingRollingEvidenceSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingEvidenceSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingEvidenceSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingEvidenceSlotModifier for TieredRollingEvidenceSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingEvidenceSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingEvidenceSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingEvidenceSlotModifier for FlaggedRollingEvidenceSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingEvidenceSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling evidence slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingEvidenceSlotEngine {
    pub steps: Vec<Rc<dyn RollingEvidenceSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingEvidenceSlotEngine {
    pub fn evaluate(&mut self, item: &RollingEvidenceSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingEvidenceSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_evidence_slot_engine(flags: &flags::Registry) -> RollingEvidenceSlotEngine {
    RollingEvidenceSlotEngine {
        steps: vec![
            Rc::new(CapRollingEvidenceSlotAdjuster { cap: 44986.0 }) as Rc<dyn RollingEvidenceSlotModifier>,
            Rc::new(RoundingRollingEvidenceSlotAdjuster { step: 0.05 }),
            Rc::new(TieredRollingEvidenceSlotAdjuster {
                bounds: vec![216.0, 2782.0, 4492.0, 4617.0],
                rates: vec![0.86, 1.15, 1.0, 0.95],
            }),
            Rc::new(FlaggedRollingEvidenceSlotAdjuster {
                flags: flags.clone(),
                flag: "rolling-evidence-slot-tiered".to_string(),
                percent: 7.8,
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 44211.1,
    }
}
