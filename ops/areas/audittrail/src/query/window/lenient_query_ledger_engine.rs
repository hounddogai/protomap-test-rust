use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient query ledger calculation.
pub trait LenientQueryLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientQueryLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientQueryLedgerAdjuster {
    pub step: f64,
}

impl LenientQueryLedgerModifier for RoundingLenientQueryLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientQueryLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientQueryLedgerAdjuster {
    pub percent: f64,
}

impl LenientQueryLedgerModifier for PercentLenientQueryLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientQueryLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryLedgerRatioOffsetAdjuster {
    pub weight: f64,
}

impl LenientQueryLedgerModifier for LenientQueryLedgerRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &LenientQueryLedger, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientQueryLedgerAdjuster {
    pub cap: f64,
}

impl LenientQueryLedgerModifier for CapLenientQueryLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientQueryLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient query ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientQueryLedgerEngine {
    pub steps: Vec<Rc<dyn LenientQueryLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientQueryLedgerEngine {
    pub fn evaluate(&mut self, item: &LenientQueryLedger) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientQueryLedgerModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientQueryLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_query_ledger_engine() -> LenientQueryLedgerEngine {
    LenientQueryLedgerEngine {
        steps: vec![
            Rc::new(RoundingLenientQueryLedgerAdjuster { step: 5.0 }) as Rc<dyn LenientQueryLedgerModifier>,
            Rc::new(PercentLenientQueryLedgerAdjuster { percent: -7.4 }),
            Rc::new(LenientQueryLedgerRatioOffsetAdjuster { weight: 0.47 }),
            Rc::new(CapLenientQueryLedgerAdjuster { cap: 27523.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 20575.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_query_ledger_engine();
        let item = new_lenient_query_ledger("code-19", "label-767");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
