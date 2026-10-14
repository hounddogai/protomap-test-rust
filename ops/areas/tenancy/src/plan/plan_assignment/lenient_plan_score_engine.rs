use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the lenient plan score calculation.
pub trait LenientPlanScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPlanScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientPlanScoreAdjuster {
    pub cap: f64,
}

impl LenientPlanScoreModifier for CapLenientPlanScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientPlanScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanScoreQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl LenientPlanScoreModifier for LenientPlanScoreQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &LenientPlanScore, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientPlanScoreAdjuster {
    pub minimum: f64,
}

impl LenientPlanScoreModifier for MinimumLenientPlanScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientPlanScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientPlanScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientPlanScoreModifier for TieredLenientPlanScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientPlanScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient plan score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPlanScoreEngine {
    pub steps: Vec<Rc<dyn LenientPlanScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPlanScoreEngine {
    pub fn price(&mut self, item: &LenientPlanScore) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientPlanScoreModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientPlanScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_plan_score_engine() -> LenientPlanScoreEngine {
    LenientPlanScoreEngine {
        steps: vec![
            Rc::new(CapLenientPlanScoreAdjuster { cap: 32572.0 }) as Rc<dyn LenientPlanScoreModifier>,
            Rc::new(LenientPlanScoreQuotaUnitsOffsetAdjuster { weight: 1.32 }),
            Rc::new(MinimumLenientPlanScoreAdjuster { minimum: 24.3 }),
            Rc::new(TieredLenientPlanScoreAdjuster {
                bounds: vec![1226.0, 3091.0, 3752.0, 9024.0],
                rates: vec![1.04, 1.05, 0.88, 0.95],
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 73978.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_plan_score_engine();
        let item = new_lenient_plan_score("code-154", "name-880");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
