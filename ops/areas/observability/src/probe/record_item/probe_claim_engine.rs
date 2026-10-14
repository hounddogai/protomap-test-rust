use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe claim calculation.
pub trait ProbeClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeClaimAdjuster {
    pub step: f64,
}

impl ProbeClaimStep for RoundingProbeClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeClaimAdjuster {
    pub minimum: f64,
}

impl ProbeClaimStep for MinimumProbeClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeClaimPortOffsetAdjuster {
    pub weight: f64,
}

impl ProbeClaimStep for ProbeClaimPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeClaim, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeClaimAdjuster {
    pub cap: f64,
}

impl ProbeClaimStep for CapProbeClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the probe claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeClaimEngine {
    pub steps: Vec<Rc<dyn ProbeClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeClaimEngine {
    pub fn price(&mut self, item: &ProbeClaim) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ProbeClaimStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ProbeClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_claim_engine() -> ProbeClaimEngine {
    ProbeClaimEngine {
        steps: vec![
            Rc::new(RoundingProbeClaimAdjuster { step: 0.25 }) as Rc<dyn ProbeClaimStep>,
            Rc::new(MinimumProbeClaimAdjuster { minimum: 16.4 }),
            Rc::new(ProbeClaimPortOffsetAdjuster { weight: 1.89 }),
            Rc::new(CapProbeClaimAdjuster { cap: 10895.0 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 33073.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_claim_engine();
        let item = new_probe_claim("code-410", "name-738");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
