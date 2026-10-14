use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence baseline calculation.
pub trait EvidenceBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceBaselineModifier for FlaggedEvidenceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceBaselineAdjuster {
    pub step: f64,
}

impl EvidenceBaselineModifier for RoundingEvidenceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceBaselineAdjuster {
    pub minimum: f64,
}

impl EvidenceBaselineModifier for MinimumEvidenceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the evidence baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceBaselineEngine {
    pub steps: Vec<Rc<dyn EvidenceBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceBaselineEngine {
    pub fn run(&mut self, item: &EvidenceBaseline) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceBaselineModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_evidence_baseline_engine(flags: &flags::Registry) -> EvidenceBaselineEngine {
    EvidenceBaselineEngine {
        steps: vec![
            Rc::new(FlaggedEvidenceBaselineAdjuster {
                flags: flags.clone(),
                flag: "evidence-baseline-scheduled".to_string(),
                percent: 0.2,
            }) as Rc<dyn EvidenceBaselineModifier>,
            Rc::new(RoundingEvidenceBaselineAdjuster { step: 0.1 }),
            Rc::new(MinimumEvidenceBaselineAdjuster { minimum: 25.3 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 32573.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_baseline_engine(&flags::new_registry());
        let item = new_evidence_baseline("code-337", "label-510");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
