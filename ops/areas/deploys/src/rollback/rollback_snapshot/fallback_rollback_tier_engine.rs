use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback rollback tier calculation.
pub trait FallbackRollbackTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRollbackTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackRollbackTierAdjuster {
    pub minimum: f64,
}

impl FallbackRollbackTierAdjuster for MinimumFallbackRollbackTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackRollbackTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackRollbackTierAdjuster {
    pub cap: f64,
}

impl FallbackRollbackTierAdjuster for CapFallbackRollbackTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackRollbackTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackTierWeightOffsetAdjuster {
    pub weight: f64,
}

impl FallbackRollbackTierAdjuster for FallbackRollbackTierWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackRollbackTier, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackRollbackTierAdjuster {
    pub step: f64,
}

impl FallbackRollbackTierAdjuster for RoundingFallbackRollbackTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackRollbackTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackRollbackTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackRollbackTierAdjuster for TieredFallbackRollbackTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackRollbackTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentFallbackRollbackTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackRollbackTierAdjuster for SegmentFallbackRollbackTierAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &FallbackRollbackTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback rollback tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRollbackTierEngine {
    pub steps: Vec<Rc<dyn FallbackRollbackTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRollbackTierEngine {
    pub fn evaluate(&mut self, item: &FallbackRollbackTier) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRollbackTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_rollback_tier_engine() -> FallbackRollbackTierEngine {
    FallbackRollbackTierEngine {
        steps: vec![
            Rc::new(MinimumFallbackRollbackTierAdjuster { minimum: 31.3 }) as Rc<dyn FallbackRollbackTierAdjuster>,
            Rc::new(CapFallbackRollbackTierAdjuster { cap: 7279.0 }),
            Rc::new(FallbackRollbackTierWeightOffsetAdjuster { weight: 1.46 }),
            Rc::new(RoundingFallbackRollbackTierAdjuster { step: 0.05 }),
            Rc::new(TieredFallbackRollbackTierAdjuster {
                bounds: vec![1753.0, 9004.0],
                rates: vec![1.06, 1.09],
            }),
            Rc::new(SegmentFallbackRollbackTierAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.16),
                    ("north".to_string(), 0.72),
                    ("west".to_string(), 1.28),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 24277.3,
    }
}
