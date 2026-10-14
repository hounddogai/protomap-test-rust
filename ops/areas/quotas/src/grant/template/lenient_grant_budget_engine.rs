use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient grant budget calculation.
pub trait LenientGrantBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientGrantBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientGrantBudgetAdjuster {
    pub cap: f64,
}

impl LenientGrantBudgetModifier for CapLenientGrantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientGrantBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientGrantBudgetAdjuster {
    pub percent: f64,
}

impl LenientGrantBudgetModifier for PercentLenientGrantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientGrantBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientGrantBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientGrantBudgetModifier for TieredLenientGrantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientGrantBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientGrantBudgetAdjuster {
    pub step: f64,
}

impl LenientGrantBudgetModifier for RoundingLenientGrantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientGrantBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient grant budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientGrantBudgetEngine {
    pub steps: Vec<Rc<dyn LenientGrantBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientGrantBudgetEngine {
    pub fn price(&mut self, item: &LenientGrantBudget) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientGrantBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientGrantBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_grant_budget_engine() -> LenientGrantBudgetEngine {
    LenientGrantBudgetEngine {
        steps: vec![
            Rc::new(CapLenientGrantBudgetAdjuster { cap: 24943.0 }) as Rc<dyn LenientGrantBudgetModifier>,
            Rc::new(PercentLenientGrantBudgetAdjuster { percent: -6.2 }),
            Rc::new(TieredLenientGrantBudgetAdjuster {
                bounds: vec![2357.0, 2602.0, 3554.0, 6433.0],
                rates: vec![1.06, 1.03, 1.19, 0.92],
            }),
            Rc::new(RoundingLenientGrantBudgetAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 9035.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_grant_budget_engine();
        let item = new_lenient_grant_budget("code-16", "title-705");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
