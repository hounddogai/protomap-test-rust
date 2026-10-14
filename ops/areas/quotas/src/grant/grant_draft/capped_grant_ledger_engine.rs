use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped grant ledger calculation.
pub trait CappedGrantLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedGrantLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedGrantLedgerAdjuster {
    pub minimum: f64,
}

impl CappedGrantLedgerAdjuster for MinimumCappedGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedGrantLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelCappedGrantLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedGrantLedgerAdjuster for ChannelCappedGrantLedgerAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &CappedGrantLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantLedgerStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl CappedGrantLedgerAdjuster for CappedGrantLedgerStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &CappedGrantLedger, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedGrantLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedGrantLedgerAdjuster for TieredCappedGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedGrantLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped grant ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedGrantLedgerEngine {
    pub steps: Vec<Rc<dyn CappedGrantLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedGrantLedgerEngine {
    pub fn run(&mut self, item: &CappedGrantLedger) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedGrantLedgerAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedGrantLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_grant_ledger_engine() -> CappedGrantLedgerEngine {
    CappedGrantLedgerEngine {
        steps: vec![
            Rc::new(MinimumCappedGrantLedgerAdjuster { minimum: 0.3 }) as Rc<dyn CappedGrantLedgerAdjuster>,
            Rc::new(ChannelCappedGrantLedgerAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.26)]),
            }),
            Rc::new(CappedGrantLedgerStartsAtOffsetAdjuster { weight: 0.17 }),
            Rc::new(TieredCappedGrantLedgerAdjuster {
                bounds: vec![3520.0, 4217.0, 5441.0, 6198.0],
                rates: vec![0.98, 0.95, 1.19, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 78035.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_grant_ledger_engine();
        let item = new_capped_grant_ledger("code-368", "label-740");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
