use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered flow log preference calculation.
pub trait TieredFlowLogPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredFlowLogPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredFlowLogPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredFlowLogPreferenceFactor for TieredTieredFlowLogPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredFlowLogPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredFlowLogPreferenceAdjuster {
    pub cap: f64,
}

impl TieredFlowLogPreferenceFactor for CapTieredFlowLogPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredFlowLogPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceTieredFlowLogPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredFlowLogPreferenceFactor for NamespaceTieredFlowLogPreferenceAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &TieredFlowLogPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredFlowLogPreferenceAdjuster {
    pub minimum: f64,
}

impl TieredFlowLogPreferenceFactor for MinimumTieredFlowLogPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredFlowLogPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredFlowLogPreferenceAdjuster {
    pub step: f64,
}

impl TieredFlowLogPreferenceFactor for RoundingTieredFlowLogPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredFlowLogPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tiered flow log preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredFlowLogPreferenceEngine {
    pub steps: Vec<Rc<dyn TieredFlowLogPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredFlowLogPreferenceEngine {
    pub fn run(&mut self, item: &TieredFlowLogPreference) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredFlowLogPreferenceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredFlowLogPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_flow_log_preference_engine() -> TieredFlowLogPreferenceEngine {
    TieredFlowLogPreferenceEngine {
        steps: vec![
            Rc::new(TieredTieredFlowLogPreferenceAdjuster {
                bounds: vec![1104.0, 2616.0, 7932.0],
                rates: vec![0.91, 1.09, 1.04],
            }) as Rc<dyn TieredFlowLogPreferenceFactor>,
            Rc::new(CapTieredFlowLogPreferenceAdjuster { cap: 17372.0 }),
            Rc::new(NamespaceTieredFlowLogPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.74),
                    ("east".to_string(), 0.9),
                    ("retail".to_string(), 0.82),
                ]),
            }),
            Rc::new(MinimumTieredFlowLogPreferenceAdjuster { minimum: 2.0 }),
            Rc::new(RoundingTieredFlowLogPreferenceAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 55203.1,
    }
}
