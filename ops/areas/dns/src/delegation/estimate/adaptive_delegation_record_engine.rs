use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive delegation record calculation.
pub trait AdaptiveDelegationRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveDelegationRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveDelegationRecordAdjuster {
    pub percent: f64,
}

impl AdaptiveDelegationRecordStep for PercentAdaptiveDelegationRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveDelegationRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveDelegationRecordAdjuster {
    pub minimum: f64,
}

impl AdaptiveDelegationRecordStep for MinimumAdaptiveDelegationRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveDelegationRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveDelegationRecordAdjuster {
    pub cap: f64,
}

impl AdaptiveDelegationRecordStep for CapAdaptiveDelegationRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveDelegationRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveDelegationRecordAdjuster {
    pub step: f64,
}

impl AdaptiveDelegationRecordStep for RoundingAdaptiveDelegationRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveDelegationRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive delegation record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveDelegationRecordEngine {
    pub steps: Vec<Rc<dyn AdaptiveDelegationRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveDelegationRecordEngine {
    pub fn price(&mut self, item: &AdaptiveDelegationRecord) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveDelegationRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveDelegationRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_delegation_record_engine() -> AdaptiveDelegationRecordEngine {
    AdaptiveDelegationRecordEngine {
        steps: vec![
            Rc::new(PercentAdaptiveDelegationRecordAdjuster { percent: -7.6 }) as Rc<dyn AdaptiveDelegationRecordStep>,
            Rc::new(MinimumAdaptiveDelegationRecordAdjuster { minimum: 31.6 }),
            Rc::new(CapAdaptiveDelegationRecordAdjuster { cap: 38525.0 }),
            Rc::new(RoundingAdaptiveDelegationRecordAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 28913.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_delegation_record_engine();
        let item = new_adaptive_delegation_record("code-774", "name-437");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
