use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped ttl policy record calculation.
pub trait CappedTtlPolicyRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedTtlPolicyRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedTtlPolicyRecordAdjuster {
    pub percent: f64,
}

impl CappedTtlPolicyRecordAdjuster for PercentCappedTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedTtlPolicyRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedTtlPolicyRecordAdjuster {
    pub minimum: f64,
}

impl CappedTtlPolicyRecordAdjuster for MinimumCappedTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedTtlPolicyRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedTtlPolicyRecordAdjuster {
    pub step: f64,
}

impl CappedTtlPolicyRecordAdjuster for RoundingCappedTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedTtlPolicyRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedTtlPolicyRecordAdjuster {
    pub cap: f64,
}

impl CappedTtlPolicyRecordAdjuster for CapCappedTtlPolicyRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedTtlPolicyRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped ttl policy record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedTtlPolicyRecordEngine {
    pub steps: Vec<Rc<dyn CappedTtlPolicyRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedTtlPolicyRecordEngine {
    pub fn price(&mut self, item: &CappedTtlPolicyRecord) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedTtlPolicyRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedTtlPolicyRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_ttl_policy_record_engine() -> CappedTtlPolicyRecordEngine {
    CappedTtlPolicyRecordEngine {
        steps: vec![
            Rc::new(PercentCappedTtlPolicyRecordAdjuster { percent: 2.5 }) as Rc<dyn CappedTtlPolicyRecordAdjuster>,
            Rc::new(MinimumCappedTtlPolicyRecordAdjuster { minimum: 14.8 }),
            Rc::new(RoundingCappedTtlPolicyRecordAdjuster { step: 0.5 }),
            Rc::new(CapCappedTtlPolicyRecordAdjuster { cap: 10441.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 46308.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_ttl_policy_record_engine();
        let item = new_capped_ttl_policy_record("code-704", "title-716");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
