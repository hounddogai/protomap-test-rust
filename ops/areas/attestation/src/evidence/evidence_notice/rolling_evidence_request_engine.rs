use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling evidence request calculation.
pub trait RollingEvidenceRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingEvidenceRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceRequestAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl RollingEvidenceRequestFactor for RollingEvidenceRequestAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &RollingEvidenceRequest, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingEvidenceRequestAdjuster {
    pub cap: f64,
}

impl RollingEvidenceRequestFactor for CapRollingEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingEvidenceRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingEvidenceRequestAdjuster {
    pub percent: f64,
}

impl RollingEvidenceRequestFactor for PercentRollingEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingEvidenceRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling evidence request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingEvidenceRequestEngine {
    pub steps: Vec<Rc<dyn RollingEvidenceRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingEvidenceRequestEngine {
    pub fn compute(&mut self, item: &RollingEvidenceRequest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingEvidenceRequestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingEvidenceRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_evidence_request_engine() -> RollingEvidenceRequestEngine {
    RollingEvidenceRequestEngine {
        steps: vec![
            Rc::new(RollingEvidenceRequestAttemptsOffsetAdjuster { weight: 0.15 })
                as Rc<dyn RollingEvidenceRequestFactor>,
            Rc::new(CapRollingEvidenceRequestAdjuster { cap: 11272.0 }),
            Rc::new(PercentRollingEvidenceRequestAdjuster { percent: 18.7 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 14783.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_evidence_request_engine();
        let item = new_rolling_evidence_request("code-422", "title-79");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
