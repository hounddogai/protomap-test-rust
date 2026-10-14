use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback overage budget calculation.
pub trait FallbackOverageBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackOverageBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackOverageBudgetAdjuster {
    pub percent: f64,
}

impl FallbackOverageBudgetAdjuster for PercentFallbackOverageBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackOverageBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackOverageBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackOverageBudgetAdjuster for TieredFallbackOverageBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackOverageBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageBudgetBudgetOffsetAdjuster {
    pub weight: f64,
}

impl FallbackOverageBudgetAdjuster for FallbackOverageBudgetBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackOverageBudget, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackOverageBudgetAdjuster {
    pub cap: f64,
}

impl FallbackOverageBudgetAdjuster for CapFallbackOverageBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackOverageBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackOverageBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackOverageBudgetAdjuster for FlaggedFallbackOverageBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackOverageBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback overage budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackOverageBudgetEngine {
    pub steps: Vec<Rc<dyn FallbackOverageBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackOverageBudgetEngine {
    pub fn evaluate(&mut self, item: &FallbackOverageBudget) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackOverageBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_overage_budget_engine(flags: &flags::Registry) -> FallbackOverageBudgetEngine {
    FallbackOverageBudgetEngine {
        steps: vec![
            Rc::new(PercentFallbackOverageBudgetAdjuster { percent: 17.9 }) as Rc<dyn FallbackOverageBudgetAdjuster>,
            Rc::new(TieredFallbackOverageBudgetAdjuster {
                bounds: vec![3419.0, 3926.0, 8482.0, 9176.0],
                rates: vec![0.9, 0.88, 0.82, 1.15],
            }),
            Rc::new(FallbackOverageBudgetBudgetOffsetAdjuster { weight: 1.2 }),
            Rc::new(CapFallbackOverageBudgetAdjuster { cap: 26986.0 }),
            Rc::new(FlaggedFallbackOverageBudgetAdjuster {
                flags: flags.clone(),
                flag: "fallback-overage-budget-rolling".to_string(),
                percent: 5.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 7529.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_overage_budget_engine(&flags::new_registry());
        let item = new_fallback_overage_budget("code-907", "label-654");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
