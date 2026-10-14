use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback suspension budget calculation.
pub trait FallbackSuspensionBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackSuspensionBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackSuspensionBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackSuspensionBudgetAdjuster for FlaggedFallbackSuspensionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackSuspensionBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackSuspensionBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackSuspensionBudgetAdjuster for TieredFallbackSuspensionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackSuspensionBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackSuspensionBudgetAdjuster {
    pub step: f64,
}

impl FallbackSuspensionBudgetAdjuster for RoundingFallbackSuspensionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackSuspensionBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionBudgetWeightOffsetAdjuster {
    pub weight: f64,
}

impl FallbackSuspensionBudgetAdjuster for FallbackSuspensionBudgetWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackSuspensionBudget, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the fallback suspension budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackSuspensionBudgetEngine {
    pub steps: Vec<Rc<dyn FallbackSuspensionBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackSuspensionBudgetEngine {
    pub fn price(&mut self, item: &FallbackSuspensionBudget) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackSuspensionBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackSuspensionBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_suspension_budget_engine(flags: &flags::Registry) -> FallbackSuspensionBudgetEngine {
    FallbackSuspensionBudgetEngine {
        steps: vec![
            Rc::new(FlaggedFallbackSuspensionBudgetAdjuster {
                flags: flags.clone(),
                flag: "fallback-suspension-budget-pending".to_string(),
                percent: -9.8,
            }) as Rc<dyn FallbackSuspensionBudgetAdjuster>,
            Rc::new(TieredFallbackSuspensionBudgetAdjuster {
                bounds: vec![4232.0, 5250.0, 5709.0, 7416.0],
                rates: vec![0.97, 0.95, 0.82, 0.84],
            }),
            Rc::new(RoundingFallbackSuspensionBudgetAdjuster { step: 0.25 }),
            Rc::new(FallbackSuspensionBudgetWeightOffsetAdjuster { weight: 0.71 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 74895.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_suspension_budget_engine(&flags::new_registry());
        let item = new_fallback_suspension_budget("code-50", "label-85");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
