use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient dashboard ledger calculation.
pub trait LenientDashboardLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientDashboardLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientDashboardLedgerAdjuster {
    pub step: f64,
}

impl LenientDashboardLedgerStep for RoundingLenientDashboardLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientDashboardLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientDashboardLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientDashboardLedgerStep for LabelLenientDashboardLedgerAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientDashboardLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientDashboardLedgerAdjuster {
    pub percent: f64,
}

impl LenientDashboardLedgerStep for PercentLenientDashboardLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientDashboardLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientDashboardLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientDashboardLedgerStep for TieredLenientDashboardLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientDashboardLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient dashboard ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientDashboardLedgerEngine {
    pub steps: Vec<Rc<dyn LenientDashboardLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientDashboardLedgerEngine {
    pub fn run(&mut self, item: &LenientDashboardLedger) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientDashboardLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientDashboardLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_dashboard_ledger_engine() -> LenientDashboardLedgerEngine {
    LenientDashboardLedgerEngine {
        steps: vec![
            Rc::new(RoundingLenientDashboardLedgerAdjuster { step: 0.25 }) as Rc<dyn LenientDashboardLedgerStep>,
            Rc::new(LabelLenientDashboardLedgerAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.1), ("online".to_string(), 1.32)]),
            }),
            Rc::new(PercentLenientDashboardLedgerAdjuster { percent: 23.0 }),
            Rc::new(TieredLenientDashboardLedgerAdjuster {
                bounds: vec![2583.0, 8331.0],
                rates: vec![0.89, 1.06],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 30499.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_dashboard_ledger_engine();
        let item = new_lenient_dashboard_ledger("code-12", "label-532");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
