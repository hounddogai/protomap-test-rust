use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted escalation tier calculation.
pub trait WeightedEscalationTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedEscalationTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionWeightedEscalationTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedEscalationTierAdjuster for RegionWeightedEscalationTierAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &WeightedEscalationTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedEscalationTierAdjuster {
    pub step: f64,
}

impl WeightedEscalationTierAdjuster for RoundingWeightedEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedEscalationTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedEscalationTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedEscalationTierAdjuster for TieredWeightedEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedEscalationTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTierQuantityOffsetAdjuster {
    pub weight: f64,
}

impl WeightedEscalationTierAdjuster for WeightedEscalationTierQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedEscalationTier, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedEscalationTierAdjuster {
    pub cap: f64,
}

impl WeightedEscalationTierAdjuster for CapWeightedEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedEscalationTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted escalation tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedEscalationTierEngine {
    pub steps: Vec<Rc<dyn WeightedEscalationTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedEscalationTierEngine {
    pub fn price(&mut self, item: &WeightedEscalationTier) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedEscalationTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_escalation_tier_engine() -> WeightedEscalationTierEngine {
    WeightedEscalationTierEngine {
        steps: vec![
            Rc::new(RegionWeightedEscalationTierAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.95)]),
            }) as Rc<dyn WeightedEscalationTierAdjuster>,
            Rc::new(RoundingWeightedEscalationTierAdjuster { step: 0.5 }),
            Rc::new(TieredWeightedEscalationTierAdjuster {
                bounds: vec![714.0, 3035.0, 7513.0],
                rates: vec![1.09, 1.14, 1.0],
            }),
            Rc::new(WeightedEscalationTierQuantityOffsetAdjuster { weight: 0.52 }),
            Rc::new(CapWeightedEscalationTierAdjuster { cap: 27911.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 65223.6,
    }
}
