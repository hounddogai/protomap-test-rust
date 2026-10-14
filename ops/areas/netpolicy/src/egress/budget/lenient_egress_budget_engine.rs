use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient egress budget calculation.
pub trait LenientEgressBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientEgressBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientEgressBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientEgressBudgetModifier for FlaggedLenientEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientEgressBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientEgressBudgetAdjuster {
    pub percent: f64,
}

impl LenientEgressBudgetModifier for PercentLenientEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientEgressBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressBudgetMarginOffsetAdjuster {
    pub weight: f64,
}

impl LenientEgressBudgetModifier for LenientEgressBudgetMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &LenientEgressBudget, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientEgressBudgetAdjuster {
    pub cap: f64,
}

impl LenientEgressBudgetModifier for CapLenientEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientEgressBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientEgressBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientEgressBudgetModifier for TieredLenientEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientEgressBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient egress budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientEgressBudgetEngine {
    pub steps: Vec<Rc<dyn LenientEgressBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientEgressBudgetEngine {
    pub fn compute(&mut self, item: &LenientEgressBudget) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientEgressBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_egress_budget_engine(flags: &flags::Registry) -> LenientEgressBudgetEngine {
    LenientEgressBudgetEngine {
        steps: vec![
            Rc::new(FlaggedLenientEgressBudgetAdjuster {
                flags: flags.clone(),
                flag: "lenient-egress-budget-legacy".to_string(),
                percent: 1.8,
            }) as Rc<dyn LenientEgressBudgetModifier>,
            Rc::new(PercentLenientEgressBudgetAdjuster { percent: 23.8 }),
            Rc::new(LenientEgressBudgetMarginOffsetAdjuster { weight: 1.27 }),
            Rc::new(CapLenientEgressBudgetAdjuster { cap: 45858.0 }),
            Rc::new(TieredLenientEgressBudgetAdjuster {
                bounds: vec![2583.0, 3384.0, 6818.0, 9228.0],
                rates: vec![0.85, 1.06, 1.04, 0.8],
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 74961.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_egress_budget_engine(&flags::new_registry());
        let item = new_lenient_egress_budget("code-951", "title-909");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
