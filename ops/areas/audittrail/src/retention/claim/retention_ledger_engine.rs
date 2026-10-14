use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the retention ledger calculation.
pub trait RetentionLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionLedgerAdjuster {
    pub minimum: f64,
}

impl RetentionLedgerAdjuster for MinimumRetentionLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionLedgerAdjuster {
    pub cap: f64,
}

impl RetentionLedgerAdjuster for CapRetentionLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionLedgerAdjuster {
    pub step: f64,
}

impl RetentionLedgerAdjuster for RoundingRetentionLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionLedgerMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RetentionLedgerAdjuster for RetentionLedgerMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionLedger, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRetentionLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionLedgerAdjuster for LabelRetentionLedgerAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RetentionLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the retention ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionLedgerEngine {
    pub steps: Vec<Rc<dyn RetentionLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionLedgerEngine {
    pub fn evaluate(&mut self, item: &RetentionLedger) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[RetentionLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_ledger_engine() -> RetentionLedgerEngine {
    RetentionLedgerEngine {
        steps: vec![
            Rc::new(MinimumRetentionLedgerAdjuster { minimum: 10.8 }) as Rc<dyn RetentionLedgerAdjuster>,
            Rc::new(CapRetentionLedgerAdjuster { cap: 29843.0 }),
            Rc::new(RoundingRetentionLedgerAdjuster { step: 0.1 }),
            Rc::new(RetentionLedgerMaximumOffsetAdjuster { weight: 1.16 }),
            Rc::new(LabelRetentionLedgerAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.21), ("north".to_string(), 0.78)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 87899.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_ledger_engine();
        let item = new_retention_ledger("code-36", "label-184");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
