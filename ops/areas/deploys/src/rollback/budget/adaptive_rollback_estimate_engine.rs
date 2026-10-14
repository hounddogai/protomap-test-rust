use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive rollback estimate calculation.
pub trait AdaptiveRollbackEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRollbackEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRollbackEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRollbackEstimateModifier for TieredAdaptiveRollbackEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRollbackEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAdaptiveRollbackEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRollbackEstimateModifier for LocaleAdaptiveRollbackEstimateAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AdaptiveRollbackEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveRollbackEstimateAdjuster {
    pub cap: f64,
}

impl AdaptiveRollbackEstimateModifier for CapAdaptiveRollbackEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveRollbackEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveRollbackEstimateAdjuster {
    pub step: f64,
}

impl AdaptiveRollbackEstimateModifier for RoundingAdaptiveRollbackEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveRollbackEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRollbackEstimateAdjuster {
    pub minimum: f64,
}

impl AdaptiveRollbackEstimateModifier for MinimumAdaptiveRollbackEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRollbackEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive rollback estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRollbackEstimateEngine {
    pub steps: Vec<Rc<dyn AdaptiveRollbackEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRollbackEstimateEngine {
    pub fn price(&mut self, item: &AdaptiveRollbackEstimate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRollbackEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_rollback_estimate_engine() -> AdaptiveRollbackEstimateEngine {
    AdaptiveRollbackEstimateEngine {
        steps: vec![
            Rc::new(TieredAdaptiveRollbackEstimateAdjuster {
                bounds: vec![6782.0, 8388.0, 8966.0, 9919.0],
                rates: vec![1.01, 0.86, 1.07, 0.84],
            }) as Rc<dyn AdaptiveRollbackEstimateModifier>,
            Rc::new(LocaleAdaptiveRollbackEstimateAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.71), ("online".to_string(), 0.95)]),
            }),
            Rc::new(CapAdaptiveRollbackEstimateAdjuster { cap: 26536.0 }),
            Rc::new(RoundingAdaptiveRollbackEstimateAdjuster { step: 5.0 }),
            Rc::new(MinimumAdaptiveRollbackEstimateAdjuster { minimum: 22.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 23928.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_rollback_estimate_engine();
        let item = new_adaptive_rollback_estimate("code-593", "label-285");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
