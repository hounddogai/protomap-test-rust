use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the nonce ledger calculation.
pub trait NonceLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceLedgerAdjuster {
    pub step: f64,
}

impl NonceLedgerFactor for RoundingNonceLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceLedgerFactor for TieredNonceLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceLedgerAdjuster {
    pub percent: f64,
}

impl NonceLedgerFactor for PercentNonceLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceLedgerScoreOffsetAdjuster {
    pub weight: f64,
}

impl NonceLedgerFactor for NonceLedgerScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &NonceLedger, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the nonce ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceLedgerEngine {
    pub steps: Vec<Rc<dyn NonceLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceLedgerEngine {
    pub fn price(&mut self, item: &NonceLedger) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceLedgerFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_nonce_ledger_engine() -> NonceLedgerEngine {
    NonceLedgerEngine {
        steps: vec![
            Rc::new(RoundingNonceLedgerAdjuster { step: 5.0 }) as Rc<dyn NonceLedgerFactor>,
            Rc::new(TieredNonceLedgerAdjuster {
                bounds: vec![4196.0, 8270.0],
                rates: vec![0.92, 0.89],
            }),
            Rc::new(PercentNonceLedgerAdjuster { percent: 19.7 }),
            Rc::new(NonceLedgerScoreOffsetAdjuster { weight: 0.69 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 27852.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_ledger_engine();
        let item = new_nonce_ledger("code-712", "title-498");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
