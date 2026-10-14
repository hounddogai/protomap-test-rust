use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default nonce target calculation.
pub trait DefaultNonceTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultNonceTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultNonceTargetAdjuster {
    pub step: f64,
}

impl DefaultNonceTargetStep for RoundingDefaultNonceTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultNonceTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultNonceTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultNonceTargetStep for LabelDefaultNonceTargetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultNonceTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultNonceTargetAdjuster {
    pub cap: f64,
}

impl DefaultNonceTargetStep for CapDefaultNonceTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultNonceTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultNonceTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultNonceTargetStep for TieredDefaultNonceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultNonceTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNonceTargetAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl DefaultNonceTargetStep for DefaultNonceTargetAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultNonceTarget, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultNonceTargetAdjuster {
    pub minimum: f64,
}

impl DefaultNonceTargetStep for MinimumDefaultNonceTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultNonceTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the default nonce target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultNonceTargetEngine {
    pub steps: Vec<Rc<dyn DefaultNonceTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultNonceTargetEngine {
    pub fn compute(&mut self, item: &DefaultNonceTarget) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultNonceTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_nonce_target_engine() -> DefaultNonceTargetEngine {
    DefaultNonceTargetEngine {
        steps: vec![
            Rc::new(RoundingDefaultNonceTargetAdjuster { step: 0.1 }) as Rc<dyn DefaultNonceTargetStep>,
            Rc::new(LabelDefaultNonceTargetAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.08),
                    ("online".to_string(), 0.88),
                    ("retail".to_string(), 0.86),
                ]),
            }),
            Rc::new(CapDefaultNonceTargetAdjuster { cap: 46768.0 }),
            Rc::new(TieredDefaultNonceTargetAdjuster {
                bounds: vec![3339.0, 6808.0, 8587.0, 9395.0],
                rates: vec![0.94, 1.19, 1.04, 1.18],
            }),
            Rc::new(DefaultNonceTargetAttemptsOffsetAdjuster { weight: 0.68 }),
            Rc::new(MinimumDefaultNonceTargetAdjuster { minimum: 34.3 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 97474.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_nonce_target_engine();
        let item = new_default_nonce_target("code-837", "label-345");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
