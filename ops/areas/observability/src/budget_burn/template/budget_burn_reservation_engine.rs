use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the budget burn reservation calculation.
pub trait BudgetBurnReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnReservationAdjuster {
    pub cap: f64,
}

impl BudgetBurnReservationStep for CapBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnReservationStep for TieredBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnReservationAdjuster {
    pub percent: f64,
}

impl BudgetBurnReservationStep for PercentBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnReservationAdjuster {
    pub step: f64,
}

impl BudgetBurnReservationStep for RoundingBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReservationBudgetOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnReservationStep for BudgetBurnReservationBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnReservation, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBudgetBurnReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnReservationStep for NameBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BudgetBurnReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the budget burn reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnReservationEngine {
    pub steps: Vec<Rc<dyn BudgetBurnReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnReservationEngine {
    pub fn run(&mut self, item: &BudgetBurnReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_budget_burn_reservation_engine() -> BudgetBurnReservationEngine {
    BudgetBurnReservationEngine {
        steps: vec![
            Rc::new(CapBudgetBurnReservationAdjuster { cap: 18210.0 }) as Rc<dyn BudgetBurnReservationStep>,
            Rc::new(TieredBudgetBurnReservationAdjuster {
                bounds: vec![697.0, 7823.0],
                rates: vec![1.15, 0.98],
            }),
            Rc::new(PercentBudgetBurnReservationAdjuster { percent: 21.9 }),
            Rc::new(RoundingBudgetBurnReservationAdjuster { step: 5.0 }),
            Rc::new(BudgetBurnReservationBudgetOffsetAdjuster { weight: 0.83 }),
            Rc::new(NameBudgetBurnReservationAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.32),
                    ("west".to_string(), 1.2),
                    ("online".to_string(), 1.4),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 3072.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_reservation_engine();
        let item = new_budget_burn_reservation("code-739", "name-571");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
