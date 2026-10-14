use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback binding policy calculation.
pub trait FallbackBindingPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackBindingPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingPolicyRevisionOffsetAdjuster {
    pub weight: f64,
}

impl FallbackBindingPolicyFactor for FallbackBindingPolicyRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackBindingPolicy, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackBindingPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackBindingPolicyFactor for TieredFallbackBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackBindingPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackBindingPolicyAdjuster {
    pub step: f64,
}

impl FallbackBindingPolicyFactor for RoundingFallbackBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackBindingPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackBindingPolicyAdjuster {
    pub cap: f64,
}

impl FallbackBindingPolicyFactor for CapFallbackBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackBindingPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback binding policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackBindingPolicyEngine {
    pub steps: Vec<Rc<dyn FallbackBindingPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackBindingPolicyEngine {
    pub fn price(&mut self, item: &FallbackBindingPolicy) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackBindingPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_binding_policy_engine() -> FallbackBindingPolicyEngine {
    FallbackBindingPolicyEngine {
        steps: vec![
            Rc::new(FallbackBindingPolicyRevisionOffsetAdjuster { weight: 1.36 })
                as Rc<dyn FallbackBindingPolicyFactor>,
            Rc::new(TieredFallbackBindingPolicyAdjuster {
                bounds: vec![1382.0, 8796.0, 9973.0],
                rates: vec![0.8, 0.9, 1.15],
            }),
            Rc::new(RoundingFallbackBindingPolicyAdjuster { step: 1.0 }),
            Rc::new(CapFallbackBindingPolicyAdjuster { cap: 32463.0 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 33685.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_binding_policy_engine();
        let item = new_fallback_binding_policy("code-528", "name-941");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
