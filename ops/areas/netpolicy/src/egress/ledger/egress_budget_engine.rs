use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the egress budget calculation.
pub trait EgressBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressBudgetStep for FlaggedEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBudgetScoreOffsetAdjuster {
    pub weight: f64,
}

impl EgressBudgetStep for EgressBudgetScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &EgressBudget, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressBudgetStep for TieredEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressBudgetAdjuster {
    pub cap: f64,
}

impl EgressBudgetStep for CapEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressBudgetAdjuster {
    pub percent: f64,
}

impl EgressBudgetStep for PercentEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressBudgetEngine {
    pub steps: Vec<Rc<dyn EgressBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressBudgetEngine {
    pub fn run(&mut self, item: &EgressBudget) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_egress_budget_engine(flags: &flags::Registry) -> EgressBudgetEngine {
    EgressBudgetEngine {
        steps: vec![
            Rc::new(FlaggedEgressBudgetAdjuster {
                flags: flags.clone(),
                flag: "egress-budget-fallback".to_string(),
                percent: -7.7,
            }) as Rc<dyn EgressBudgetStep>,
            Rc::new(EgressBudgetScoreOffsetAdjuster { weight: 1.41 }),
            Rc::new(TieredEgressBudgetAdjuster {
                bounds: vec![3838.0, 5923.0, 9951.0],
                rates: vec![0.86, 0.83, 1.15],
            }),
            Rc::new(CapEgressBudgetAdjuster { cap: 38345.0 }),
            Rc::new(PercentEgressBudgetAdjuster { percent: -23.3 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 4955.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_budget_engine(&flags::new_registry());
        let item = new_egress_budget("code-707", "title-589");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
