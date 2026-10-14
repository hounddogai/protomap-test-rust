use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the limit ledger calculation.
pub trait LimitLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitLedgerStep for TieredLimitLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitLedgerAdjuster {
    pub step: f64,
}

impl LimitLedgerStep for RoundingLimitLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterLimitLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitLedgerStep for ClusterLimitLedgerAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &LimitLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitLedgerBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl LimitLedgerStep for LimitLedgerBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &LimitLedger, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitLedgerAdjuster {
    pub minimum: f64,
}

impl LimitLedgerStep for MinimumLimitLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the limit ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitLedgerEngine {
    pub steps: Vec<Rc<dyn LimitLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitLedgerEngine {
    pub fn price(&mut self, item: &LimitLedger) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[LimitLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_limit_ledger_engine() -> LimitLedgerEngine {
    LimitLedgerEngine {
        steps: vec![
            Rc::new(TieredLimitLedgerAdjuster {
                bounds: vec![1316.0, 6898.0],
                rates: vec![1.17, 0.95],
            }) as Rc<dyn LimitLedgerStep>,
            Rc::new(RoundingLimitLedgerAdjuster { step: 1.0 }),
            Rc::new(ClusterLimitLedgerAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.98)]),
            }),
            Rc::new(LimitLedgerBatchSizeOffsetAdjuster { weight: 1.59 }),
            Rc::new(MinimumLimitLedgerAdjuster { minimum: 22.9 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 65374.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_ledger_engine();
        let item = new_limit_ledger("code-521", "title-286");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
