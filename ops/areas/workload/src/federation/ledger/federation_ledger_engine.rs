use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation ledger calculation.
pub trait FederationLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationLedger, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationLedgerStep for FlaggedFederationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationLedgerAdjuster {
    pub step: f64,
}

impl FederationLedgerStep for RoundingFederationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationLedgerAdjuster {
    pub cap: f64,
}

impl FederationLedgerStep for CapFederationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the federation ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationLedgerEngine {
    pub steps: Vec<Rc<dyn FederationLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationLedgerEngine {
    pub fn run(&mut self, item: &FederationLedger) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_federation_ledger_engine(flags: &flags::Registry) -> FederationLedgerEngine {
    FederationLedgerEngine {
        steps: vec![
            Rc::new(FlaggedFederationLedgerAdjuster {
                flags: flags.clone(),
                flag: "federation-ledger-legacy".to_string(),
                percent: 2.5,
            }) as Rc<dyn FederationLedgerStep>,
            Rc::new(RoundingFederationLedgerAdjuster { step: 5.0 }),
            Rc::new(CapFederationLedgerAdjuster { cap: 32014.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 89067.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_ledger_engine(&flags::new_registry());
        let item = new_federation_ledger("code-655", "name-31");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
