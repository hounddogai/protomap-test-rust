use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy limit policy calculation.
pub trait LegacyLimitPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyLimitPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyLimitPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyLimitPolicyFactor for TieredLegacyLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyLimitPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyLimitPolicyAdjuster {
    pub cap: f64,
}

impl LegacyLimitPolicyFactor for CapLegacyLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyLimitPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryLegacyLimitPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyLimitPolicyFactor for CategoryLegacyLimitPolicyAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &LegacyLimitPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyLimitPolicyAdjuster {
    pub minimum: f64,
}

impl LegacyLimitPolicyFactor for MinimumLegacyLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyLimitPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPolicyGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl LegacyLimitPolicyFactor for LegacyLimitPolicyGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyLimitPolicy, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the legacy limit policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyLimitPolicyEngine {
    pub steps: Vec<Rc<dyn LegacyLimitPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyLimitPolicyEngine {
    pub fn price(&mut self, item: &LegacyLimitPolicy) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyLimitPolicyFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyLimitPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_limit_policy_engine() -> LegacyLimitPolicyEngine {
    LegacyLimitPolicyEngine {
        steps: vec![
            Rc::new(TieredLegacyLimitPolicyAdjuster {
                bounds: vec![5686.0, 8017.0],
                rates: vec![1.13, 1.18],
            }) as Rc<dyn LegacyLimitPolicyFactor>,
            Rc::new(CapLegacyLimitPolicyAdjuster { cap: 10213.0 }),
            Rc::new(CategoryLegacyLimitPolicyAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.97), ("west".to_string(), 0.76)]),
            }),
            Rc::new(MinimumLegacyLimitPolicyAdjuster { minimum: 31.6 }),
            Rc::new(LegacyLimitPolicyGraceDaysOffsetAdjuster { weight: 1.13 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 91942.3,
    }
}
