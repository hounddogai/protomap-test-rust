use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped probe agreement calculation.
pub trait CappedProbeAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedProbeAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedProbeAgreementAdjuster {
    pub step: f64,
}

impl CappedProbeAgreementModifier for RoundingCappedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedProbeAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedProbeAgreementAdjuster {
    pub cap: f64,
}

impl CappedProbeAgreementModifier for CapCappedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedProbeAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCappedProbeAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedProbeAgreementModifier for SegmentCappedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &CappedProbeAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedProbeAgreementAdjuster {
    pub percent: f64,
}

impl CappedProbeAgreementModifier for PercentCappedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedProbeAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedProbeAgreementAdjuster {
    pub minimum: f64,
}

impl CappedProbeAgreementModifier for MinimumCappedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedProbeAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped probe agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedProbeAgreementEngine {
    pub steps: Vec<Rc<dyn CappedProbeAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedProbeAgreementEngine {
    pub fn evaluate(&mut self, item: &CappedProbeAgreement) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedProbeAgreementModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedProbeAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_probe_agreement_engine() -> CappedProbeAgreementEngine {
    CappedProbeAgreementEngine {
        steps: vec![
            Rc::new(RoundingCappedProbeAgreementAdjuster { step: 1.0 }) as Rc<dyn CappedProbeAgreementModifier>,
            Rc::new(CapCappedProbeAgreementAdjuster { cap: 44206.0 }),
            Rc::new(SegmentCappedProbeAgreementAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.94)]),
            }),
            Rc::new(PercentCappedProbeAgreementAdjuster { percent: -15.8 }),
            Rc::new(MinimumCappedProbeAgreementAdjuster { minimum: 4.6 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 63974.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_probe_agreement_engine();
        let item = new_capped_probe_agreement("code-329", "label-414");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
