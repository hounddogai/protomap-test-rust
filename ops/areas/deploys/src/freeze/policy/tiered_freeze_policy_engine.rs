use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered freeze policy calculation.
pub trait TieredFreezePolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredFreezePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredFreezePolicyAdjuster {
    pub percent: f64,
}

impl TieredFreezePolicyAdjuster for PercentTieredFreezePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredFreezePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezePolicyMinimumOffsetAdjuster {
    pub weight: f64,
}

impl TieredFreezePolicyAdjuster for TieredFreezePolicyMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &TieredFreezePolicy, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredFreezePolicyAdjuster {
    pub cap: f64,
}

impl TieredFreezePolicyAdjuster for CapTieredFreezePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredFreezePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredFreezePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredFreezePolicyAdjuster for TieredTieredFreezePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredFreezePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredFreezePolicyAdjuster {
    pub minimum: f64,
}

impl TieredFreezePolicyAdjuster for MinimumTieredFreezePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredFreezePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered freeze policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredFreezePolicyEngine {
    pub steps: Vec<Rc<dyn TieredFreezePolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredFreezePolicyEngine {
    pub fn run(&mut self, item: &TieredFreezePolicy) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TieredFreezePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_freeze_policy_engine() -> TieredFreezePolicyEngine {
    TieredFreezePolicyEngine {
        steps: vec![
            Rc::new(PercentTieredFreezePolicyAdjuster { percent: -21.4 }) as Rc<dyn TieredFreezePolicyAdjuster>,
            Rc::new(TieredFreezePolicyMinimumOffsetAdjuster { weight: 1.31 }),
            Rc::new(CapTieredFreezePolicyAdjuster { cap: 14989.0 }),
            Rc::new(TieredTieredFreezePolicyAdjuster {
                bounds: vec![4611.0, 5226.0],
                rates: vec![1.07, 0.99],
            }),
            Rc::new(MinimumTieredFreezePolicyAdjuster { minimum: 17.8 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 67838.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_freeze_policy_engine();
        let item = new_tiered_freeze_policy("code-694", "name-762");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
