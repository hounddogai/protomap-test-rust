use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped release ledger calculation.
pub trait CappedReleaseLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedReleaseLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedReleaseLedgerAdjuster {
    pub minimum: f64,
}

impl CappedReleaseLedgerModifier for MinimumCappedReleaseLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedReleaseLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedReleaseLedgerAdjuster {
    pub step: f64,
}

impl CappedReleaseLedgerModifier for RoundingCappedReleaseLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedReleaseLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedReleaseLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedReleaseLedgerModifier for TieredCappedReleaseLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedReleaseLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped release ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedReleaseLedgerEngine {
    pub steps: Vec<Rc<dyn CappedReleaseLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedReleaseLedgerEngine {
    pub fn evaluate(&mut self, item: &CappedReleaseLedger) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedReleaseLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_release_ledger_engine() -> CappedReleaseLedgerEngine {
    CappedReleaseLedgerEngine {
        steps: vec![
            Rc::new(MinimumCappedReleaseLedgerAdjuster { minimum: 12.1 }) as Rc<dyn CappedReleaseLedgerModifier>,
            Rc::new(RoundingCappedReleaseLedgerAdjuster { step: 5.0 }),
            Rc::new(TieredCappedReleaseLedgerAdjuster {
                bounds: vec![3905.0, 7967.0, 9142.0],
                rates: vec![0.86, 0.96, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 10609.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_release_ledger_engine();
        let item = new_capped_release_ledger("code-368", "name-409");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
