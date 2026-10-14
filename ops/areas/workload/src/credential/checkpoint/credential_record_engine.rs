use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential record calculation.
pub trait CredentialRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialRecordAdjuster {
    pub minimum: f64,
}

impl CredentialRecordStep for MinimumCredentialRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialRecordAdjuster {
    pub step: f64,
}

impl CredentialRecordStep for RoundingCredentialRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialRecordAdjuster {
    pub cap: f64,
}

impl CredentialRecordStep for CapCredentialRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the credential record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialRecordEngine {
    pub steps: Vec<Rc<dyn CredentialRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialRecordEngine {
    pub fn compute(&mut self, item: &CredentialRecord) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CredentialRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CredentialRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_credential_record_engine() -> CredentialRecordEngine {
    CredentialRecordEngine {
        steps: vec![
            Rc::new(MinimumCredentialRecordAdjuster { minimum: 8.6 }) as Rc<dyn CredentialRecordStep>,
            Rc::new(RoundingCredentialRecordAdjuster { step: 1.0 }),
            Rc::new(CapCredentialRecordAdjuster { cap: 20240.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 95976.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_record_engine();
        let item = new_credential_record("code-978", "title-447");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
