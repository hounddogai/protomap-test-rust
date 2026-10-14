use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped budget burn ledger calculation.
pub trait CappedBudgetBurnLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedBudgetBurnLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceCappedBudgetBurnLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedBudgetBurnLedgerStep for NamespaceCappedBudgetBurnLedgerAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &CappedBudgetBurnLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedBudgetBurnLedgerAdjuster {
    pub cap: f64,
}

impl CappedBudgetBurnLedgerStep for CapCappedBudgetBurnLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedBudgetBurnLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedBudgetBurnLedgerAdjuster {
    pub minimum: f64,
}

impl CappedBudgetBurnLedgerStep for MinimumCappedBudgetBurnLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedBudgetBurnLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedBudgetBurnLedgerAdjuster {
    pub step: f64,
}

impl CappedBudgetBurnLedgerStep for RoundingCappedBudgetBurnLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedBudgetBurnLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnLedgerUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl CappedBudgetBurnLedgerStep for CappedBudgetBurnLedgerUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &CappedBudgetBurnLedger, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedBudgetBurnLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedBudgetBurnLedgerStep for TieredCappedBudgetBurnLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedBudgetBurnLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped budget burn ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnLedgerEngine {
    pub steps: Vec<Rc<dyn CappedBudgetBurnLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedBudgetBurnLedgerEngine {
    pub fn compute(&mut self, item: &CappedBudgetBurnLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedBudgetBurnLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_budget_burn_ledger_engine() -> CappedBudgetBurnLedgerEngine {
    CappedBudgetBurnLedgerEngine {
        steps: vec![
            Rc::new(NamespaceCappedBudgetBurnLedgerAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.37),
                    ("central".to_string(), 1.09),
                    ("north".to_string(), 1.13),
                ]),
            }) as Rc<dyn CappedBudgetBurnLedgerStep>,
            Rc::new(CapCappedBudgetBurnLedgerAdjuster { cap: 42383.0 }),
            Rc::new(MinimumCappedBudgetBurnLedgerAdjuster { minimum: 18.3 }),
            Rc::new(RoundingCappedBudgetBurnLedgerAdjuster { step: 0.05 }),
            Rc::new(CappedBudgetBurnLedgerUpdatedAtOffsetAdjuster { weight: 0.72 }),
            Rc::new(TieredCappedBudgetBurnLedgerAdjuster {
                bounds: vec![1039.0, 5478.0],
                rates: vec![0.97, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 51331.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_budget_burn_ledger_engine();
        let item = new_capped_budget_burn_ledger("code-718", "title-748");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
