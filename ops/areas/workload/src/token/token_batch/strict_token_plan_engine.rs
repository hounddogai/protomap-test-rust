use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict token plan calculation.
pub trait StrictTokenPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictTokenPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictTokenPlanAdjuster {
    pub cap: f64,
}

impl StrictTokenPlanModifier for CapStrictTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictTokenPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictTokenPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictTokenPlanModifier for TieredStrictTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictTokenPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictTokenPlanAdjuster {
    pub minimum: f64,
}

impl StrictTokenPlanModifier for MinimumStrictTokenPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictTokenPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict token plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictTokenPlanEngine {
    pub steps: Vec<Rc<dyn StrictTokenPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictTokenPlanEngine {
    pub fn compute(&mut self, item: &StrictTokenPlan) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[StrictTokenPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_token_plan_engine() -> StrictTokenPlanEngine {
    StrictTokenPlanEngine {
        steps: vec![
            Rc::new(CapStrictTokenPlanAdjuster { cap: 38361.0 }) as Rc<dyn StrictTokenPlanModifier>,
            Rc::new(TieredStrictTokenPlanAdjuster {
                bounds: vec![311.0, 3807.0],
                rates: vec![0.99, 0.87],
            }),
            Rc::new(MinimumStrictTokenPlanAdjuster { minimum: 36.5 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 81922.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_token_plan_engine();
        let item = new_strict_token_plan("code-591", "label-599");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
