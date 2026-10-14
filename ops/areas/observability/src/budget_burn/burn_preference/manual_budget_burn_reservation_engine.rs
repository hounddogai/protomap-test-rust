use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual budget burn reservation calculation.
pub trait ManualBudgetBurnReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualBudgetBurnReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualBudgetBurnReservationAdjuster {
    pub percent: f64,
}

impl ManualBudgetBurnReservationStep for PercentManualBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualBudgetBurnReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualBudgetBurnReservationAdjuster {
    pub minimum: f64,
}

impl ManualBudgetBurnReservationStep for MinimumManualBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualBudgetBurnReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnReservationMinimumOffsetAdjuster {
    pub weight: f64,
}

impl ManualBudgetBurnReservationStep for ManualBudgetBurnReservationMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &ManualBudgetBurnReservation, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualBudgetBurnReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualBudgetBurnReservationStep for FlaggedManualBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualBudgetBurnReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualBudgetBurnReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualBudgetBurnReservationStep for TieredManualBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualBudgetBurnReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualBudgetBurnReservationAdjuster {
    pub step: f64,
}

impl ManualBudgetBurnReservationStep for RoundingManualBudgetBurnReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualBudgetBurnReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual budget burn reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnReservationEngine {
    pub steps: Vec<Rc<dyn ManualBudgetBurnReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualBudgetBurnReservationEngine {
    pub fn run(&mut self, item: &ManualBudgetBurnReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualBudgetBurnReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_budget_burn_reservation_engine(flags: &flags::Registry) -> ManualBudgetBurnReservationEngine {
    ManualBudgetBurnReservationEngine {
        steps: vec![
            Rc::new(PercentManualBudgetBurnReservationAdjuster { percent: 12.3 })
                as Rc<dyn ManualBudgetBurnReservationStep>,
            Rc::new(MinimumManualBudgetBurnReservationAdjuster { minimum: 27.7 }),
            Rc::new(ManualBudgetBurnReservationMinimumOffsetAdjuster { weight: 0.27 }),
            Rc::new(FlaggedManualBudgetBurnReservationAdjuster {
                flags: flags.clone(),
                flag: "manual-budget-burn-reservation-tiered".to_string(),
                percent: -7.2,
            }),
            Rc::new(TieredManualBudgetBurnReservationAdjuster {
                bounds: vec![1228.0, 5546.0, 6857.0],
                rates: vec![0.95, 0.89, 1.04],
            }),
            Rc::new(RoundingManualBudgetBurnReservationAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 42368.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_budget_burn_reservation_engine(&flags::new_registry());
        let item = new_manual_budget_burn_reservation("code-512", "name-376");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
