use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending rule record calculation.
pub trait PendingRuleRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRuleRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingRuleRecordAdjuster {
    pub percent: f64,
}

impl PendingRuleRecordAdjuster for PercentPendingRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingRuleRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingRuleRecordAdjuster {
    pub minimum: f64,
}

impl PendingRuleRecordAdjuster for MinimumPendingRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingRuleRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRuleRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRuleRecordAdjuster for TieredPendingRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRuleRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending rule record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRuleRecordEngine {
    pub steps: Vec<Rc<dyn PendingRuleRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRuleRecordEngine {
    pub fn price(&mut self, item: &PendingRuleRecord) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingRuleRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingRuleRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_rule_record_engine() -> PendingRuleRecordEngine {
    PendingRuleRecordEngine {
        steps: vec![
            Rc::new(PercentPendingRuleRecordAdjuster { percent: -20.7 }) as Rc<dyn PendingRuleRecordAdjuster>,
            Rc::new(MinimumPendingRuleRecordAdjuster { minimum: 35.8 }),
            Rc::new(TieredPendingRuleRecordAdjuster {
                bounds: vec![3962.0, 7086.0, 7841.0],
                rates: vec![1.19, 0.88, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 88947.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_rule_record_engine();
        let item = new_pending_rule_record("code-241", "title-45");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
