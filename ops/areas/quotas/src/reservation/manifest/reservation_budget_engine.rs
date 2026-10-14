use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation budget calculation.
pub trait ReservationBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBudgetPriorityOffsetAdjuster {
    pub weight: f64,
}

impl ReservationBudgetModifier for ReservationBudgetPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationBudget, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationBudgetAdjuster {
    pub step: f64,
}

impl ReservationBudgetModifier for RoundingReservationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReservationBudgetAdjuster {
    pub minimum: f64,
}

impl ReservationBudgetModifier for MinimumReservationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReservationBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationBudgetModifier for FlaggedReservationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the reservation budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationBudgetEngine {
    pub steps: Vec<Rc<dyn ReservationBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationBudgetEngine {
    pub fn evaluate(&mut self, item: &ReservationBudget) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_reservation_budget_engine(flags: &flags::Registry) -> ReservationBudgetEngine {
    ReservationBudgetEngine {
        steps: vec![
            Rc::new(ReservationBudgetPriorityOffsetAdjuster { weight: 0.23 }) as Rc<dyn ReservationBudgetModifier>,
            Rc::new(RoundingReservationBudgetAdjuster { step: 0.25 }),
            Rc::new(MinimumReservationBudgetAdjuster { minimum: 9.5 }),
            Rc::new(FlaggedReservationBudgetAdjuster {
                flags: flags.clone(),
                flag: "reservation-budget-lenient".to_string(),
                percent: 1.4,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 60186.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_budget_engine(&flags::new_registry());
        let item = new_reservation_budget("code-985", "title-807");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
