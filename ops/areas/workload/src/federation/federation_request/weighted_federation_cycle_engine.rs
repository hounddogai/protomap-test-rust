use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted federation cycle calculation.
pub trait WeightedFederationCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFederationCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentWeightedFederationCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedFederationCycleAdjuster for SegmentWeightedFederationCycleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &WeightedFederationCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedFederationCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedFederationCycleAdjuster for FlaggedWeightedFederationCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedFederationCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedFederationCycleAdjuster {
    pub step: f64,
}

impl WeightedFederationCycleAdjuster for RoundingWeightedFederationCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedFederationCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedFederationCycleAdjuster {
    pub cap: f64,
}

impl WeightedFederationCycleAdjuster for CapWeightedFederationCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedFederationCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationCycleStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFederationCycleAdjuster for WeightedFederationCycleStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFederationCycle, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedFederationCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedFederationCycleAdjuster for TieredWeightedFederationCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedFederationCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted federation cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFederationCycleEngine {
    pub steps: Vec<Rc<dyn WeightedFederationCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFederationCycleEngine {
    pub fn compute(&mut self, item: &WeightedFederationCycle) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFederationCycleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFederationCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_federation_cycle_engine(flags: &flags::Registry) -> WeightedFederationCycleEngine {
    WeightedFederationCycleEngine {
        steps: vec![
            Rc::new(SegmentWeightedFederationCycleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.94),
                    ("south".to_string(), 1.34),
                    ("retail".to_string(), 0.96),
                ]),
            }) as Rc<dyn WeightedFederationCycleAdjuster>,
            Rc::new(FlaggedWeightedFederationCycleAdjuster {
                flags: flags.clone(),
                flag: "weighted-federation-cycle-fallback".to_string(),
                percent: -2.3,
            }),
            Rc::new(RoundingWeightedFederationCycleAdjuster { step: 0.5 }),
            Rc::new(CapWeightedFederationCycleAdjuster { cap: 25910.0 }),
            Rc::new(WeightedFederationCycleStartsAtOffsetAdjuster { weight: 0.82 }),
            Rc::new(TieredWeightedFederationCycleAdjuster {
                bounds: vec![2968.0, 6730.0, 7708.0],
                rates: vec![1.17, 0.96, 1.16],
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 94095.7,
    }
}
