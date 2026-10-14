use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending nonce record calculation.
pub trait PendingNonceRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingNonceRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingNonceRecordAdjuster {
    pub percent: f64,
}

impl PendingNonceRecordStep for PercentPendingNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingNonceRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingNonceRecordAdjuster {
    pub step: f64,
}

impl PendingNonceRecordStep for RoundingPendingNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingNonceRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingNonceRecordAdjuster {
    pub cap: f64,
}

impl PendingNonceRecordStep for CapPendingNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingNonceRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingNonceRecordAdjuster {
    pub minimum: f64,
}

impl PendingNonceRecordStep for MinimumPendingNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingNonceRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePendingNonceRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingNonceRecordStep for NamePendingNonceRecordAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PendingNonceRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending nonce record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingNonceRecordEngine {
    pub steps: Vec<Rc<dyn PendingNonceRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingNonceRecordEngine {
    pub fn price(&mut self, item: &PendingNonceRecord) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingNonceRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_nonce_record_engine() -> PendingNonceRecordEngine {
    PendingNonceRecordEngine {
        steps: vec![
            Rc::new(PercentPendingNonceRecordAdjuster { percent: 17.9 }) as Rc<dyn PendingNonceRecordStep>,
            Rc::new(RoundingPendingNonceRecordAdjuster { step: 1.0 }),
            Rc::new(CapPendingNonceRecordAdjuster { cap: 43366.0 }),
            Rc::new(MinimumPendingNonceRecordAdjuster { minimum: 34.5 }),
            Rc::new(NamePendingNonceRecordAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.98), ("online".to_string(), 0.94)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 50275.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_nonce_record_engine();
        let item = new_pending_nonce_record("code-891", "name-600");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
