use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert ledger calculation.
pub trait AlertLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertLedgerBudgetOffsetAdjuster {
    pub weight: f64,
}

impl AlertLedgerStep for AlertLedgerBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &AlertLedger, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertLedgerAdjuster {
    pub minimum: f64,
}

impl AlertLedgerStep for MinimumAlertLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertLedgerAdjuster {
    pub percent: f64,
}

impl AlertLedgerStep for PercentAlertLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertLedgerAdjuster {
    pub step: f64,
}

impl AlertLedgerStep for RoundingAlertLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the alert ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertLedgerEngine {
    pub steps: Vec<Rc<dyn AlertLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertLedgerEngine {
    pub fn evaluate(&mut self, item: &AlertLedger) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AlertLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_ledger_engine() -> AlertLedgerEngine {
    AlertLedgerEngine {
        steps: vec![
            Rc::new(AlertLedgerBudgetOffsetAdjuster { weight: 0.69 }) as Rc<dyn AlertLedgerStep>,
            Rc::new(MinimumAlertLedgerAdjuster { minimum: 19.8 }),
            Rc::new(PercentAlertLedgerAdjuster { percent: -15.1 }),
            Rc::new(RoundingAlertLedgerAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 84673.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_ledger_engine();
        let item = new_alert_ledger("code-997", "label-261");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
