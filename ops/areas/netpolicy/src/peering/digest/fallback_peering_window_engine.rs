use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback peering window calculation.
pub trait FallbackPeeringWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPeeringWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackPeeringWindowAdjuster {
    pub step: f64,
}

impl FallbackPeeringWindowFactor for RoundingFallbackPeeringWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackPeeringWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPeeringWindowRevisionOffsetAdjuster {
    pub weight: f64,
}

impl FallbackPeeringWindowFactor for FallbackPeeringWindowRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackPeeringWindow, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitFallbackPeeringWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackPeeringWindowFactor for UnitFallbackPeeringWindowAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &FallbackPeeringWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackPeeringWindowAdjuster {
    pub percent: f64,
}

impl FallbackPeeringWindowFactor for PercentFallbackPeeringWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackPeeringWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPeeringWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPeeringWindowFactor for TieredFallbackPeeringWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPeeringWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPeeringWindowAdjuster {
    pub minimum: f64,
}

impl FallbackPeeringWindowFactor for MinimumFallbackPeeringWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPeeringWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback peering window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPeeringWindowEngine {
    pub steps: Vec<Rc<dyn FallbackPeeringWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPeeringWindowEngine {
    pub fn evaluate(&mut self, item: &FallbackPeeringWindow) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackPeeringWindowFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[FallbackPeeringWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_peering_window_engine() -> FallbackPeeringWindowEngine {
    FallbackPeeringWindowEngine {
        steps: vec![
            Rc::new(RoundingFallbackPeeringWindowAdjuster { step: 1.0 }) as Rc<dyn FallbackPeeringWindowFactor>,
            Rc::new(FallbackPeeringWindowRevisionOffsetAdjuster { weight: 1.17 }),
            Rc::new(UnitFallbackPeeringWindowAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.98),
                    ("south".to_string(), 1.18),
                    ("retail".to_string(), 0.72),
                ]),
            }),
            Rc::new(PercentFallbackPeeringWindowAdjuster { percent: -9.5 }),
            Rc::new(TieredFallbackPeeringWindowAdjuster {
                bounds: vec![1520.0, 2241.0, 3139.0],
                rates: vec![1.15, 0.99, 1.15],
            }),
            Rc::new(MinimumFallbackPeeringWindowAdjuster { minimum: 28.8 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 30930.4,
    }
}
