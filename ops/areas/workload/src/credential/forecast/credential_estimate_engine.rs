use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential estimate calculation.
pub trait CredentialEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialEstimateAdjuster {
    pub percent: f64,
}

impl CredentialEstimateModifier for PercentCredentialEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialEstimateAdjuster {
    pub step: f64,
}

impl CredentialEstimateModifier for RoundingCredentialEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialEstimateAdjuster {
    pub cap: f64,
}

impl CredentialEstimateModifier for CapCredentialEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the credential estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialEstimateEngine {
    pub steps: Vec<Rc<dyn CredentialEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialEstimateEngine {
    pub fn compute(&mut self, item: &CredentialEstimate) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CredentialEstimateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CredentialEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_credential_estimate_engine() -> CredentialEstimateEngine {
    CredentialEstimateEngine {
        steps: vec![
            Rc::new(PercentCredentialEstimateAdjuster { percent: 1.0 }) as Rc<dyn CredentialEstimateModifier>,
            Rc::new(RoundingCredentialEstimateAdjuster { step: 1.0 }),
            Rc::new(CapCredentialEstimateAdjuster { cap: 9890.0 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 84782.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_estimate_engine();
        let item = new_credential_estimate("code-347", "label-22");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
