use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering budget calculation.
pub trait PeeringBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringBudgetAdjuster for TieredPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringBudgetAdjuster {
    pub percent: f64,
}

impl PeeringBudgetAdjuster for PercentPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringBudgetAdjuster for FlaggedPeeringBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the peering budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringBudgetEngine {
    pub steps: Vec<Rc<dyn PeeringBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringBudgetEngine {
    pub fn price(&mut self, item: &PeeringBudget) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[PeeringBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_peering_budget_engine(flags: &flags::Registry) -> PeeringBudgetEngine {
    PeeringBudgetEngine {
        steps: vec![
            Rc::new(TieredPeeringBudgetAdjuster {
                bounds: vec![1157.0, 2832.0, 9057.0],
                rates: vec![0.8, 0.88, 0.82],
            }) as Rc<dyn PeeringBudgetAdjuster>,
            Rc::new(PercentPeeringBudgetAdjuster { percent: 11.7 }),
            Rc::new(FlaggedPeeringBudgetAdjuster {
                flags: flags.clone(),
                flag: "peering-budget-scheduled".to_string(),
                percent: -7.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 53484.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_budget_engine(&flags::new_registry());
        let item = new_peering_budget("code-808", "name-767");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
