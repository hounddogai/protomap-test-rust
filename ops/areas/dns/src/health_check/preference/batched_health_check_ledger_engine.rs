use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched health check ledger calculation.
pub trait BatchedHealthCheckLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedHealthCheckLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeBatchedHealthCheckLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedHealthCheckLedgerStep for ScopeBatchedHealthCheckLedgerAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &BatchedHealthCheckLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedHealthCheckLedgerAdjuster {
    pub cap: f64,
}

impl BatchedHealthCheckLedgerStep for CapBatchedHealthCheckLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedHealthCheckLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckLedgerMarginOffsetAdjuster {
    pub weight: f64,
}

impl BatchedHealthCheckLedgerStep for BatchedHealthCheckLedgerMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedHealthCheckLedger, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedHealthCheckLedgerAdjuster {
    pub step: f64,
}

impl BatchedHealthCheckLedgerStep for RoundingBatchedHealthCheckLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedHealthCheckLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedHealthCheckLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedHealthCheckLedgerStep for FlaggedBatchedHealthCheckLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedHealthCheckLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched health check ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedHealthCheckLedgerEngine {
    pub steps: Vec<Rc<dyn BatchedHealthCheckLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedHealthCheckLedgerEngine {
    pub fn price(&mut self, item: &BatchedHealthCheckLedger) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedHealthCheckLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_health_check_ledger_engine(flags: &flags::Registry) -> BatchedHealthCheckLedgerEngine {
    BatchedHealthCheckLedgerEngine {
        steps: vec![
            Rc::new(ScopeBatchedHealthCheckLedgerAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.36),
                    ("west".to_string(), 0.99),
                    ("east".to_string(), 0.86),
                ]),
            }) as Rc<dyn BatchedHealthCheckLedgerStep>,
            Rc::new(CapBatchedHealthCheckLedgerAdjuster { cap: 46744.0 }),
            Rc::new(BatchedHealthCheckLedgerMarginOffsetAdjuster { weight: 0.81 }),
            Rc::new(RoundingBatchedHealthCheckLedgerAdjuster { step: 5.0 }),
            Rc::new(FlaggedBatchedHealthCheckLedgerAdjuster {
                flags: flags.clone(),
                flag: "batched-health-check-ledger-scheduled".to_string(),
                percent: -4.6,
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 53021.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_health_check_ledger_engine(&flags::new_registry());
        let item = new_batched_health_check_ledger("code-291", "name-469");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
