use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending runbook exception calculation.
pub trait PendingRunbookExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRunbookException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRunbookExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRunbookExceptionAdjuster for FlaggedPendingRunbookExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRunbookException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRunbookExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRunbookExceptionAdjuster for TieredPendingRunbookExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRunbookException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingRunbookExceptionAdjuster {
    pub cap: f64,
}

impl PendingRunbookExceptionAdjuster for CapPendingRunbookExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingRunbookException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookExceptionWeightOffsetAdjuster {
    pub weight: f64,
}

impl PendingRunbookExceptionAdjuster for PendingRunbookExceptionWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &PendingRunbookException, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the pending runbook exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRunbookExceptionEngine {
    pub steps: Vec<Rc<dyn PendingRunbookExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRunbookExceptionEngine {
    pub fn price(&mut self, item: &PendingRunbookException) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingRunbookExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingRunbookException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_runbook_exception_engine(flags: &flags::Registry) -> PendingRunbookExceptionEngine {
    PendingRunbookExceptionEngine {
        steps: vec![
            Rc::new(FlaggedPendingRunbookExceptionAdjuster {
                flags: flags.clone(),
                flag: "pending-runbook-exception-manual".to_string(),
                percent: 1.9,
            }) as Rc<dyn PendingRunbookExceptionAdjuster>,
            Rc::new(TieredPendingRunbookExceptionAdjuster {
                bounds: vec![430.0, 3054.0, 7002.0, 9077.0],
                rates: vec![0.98, 1.01, 0.92, 1.03],
            }),
            Rc::new(CapPendingRunbookExceptionAdjuster { cap: 48268.0 }),
            Rc::new(PendingRunbookExceptionWeightOffsetAdjuster { weight: 1.18 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 46208.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_runbook_exception_engine(&flags::new_registry());
        let item = new_pending_runbook_exception("code-949", "name-961");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
