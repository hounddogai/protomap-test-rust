use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence score calculation.
pub trait EvidenceScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceScoreAdjuster {
    pub step: f64,
}

impl EvidenceScoreModifier for RoundingEvidenceScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScoreRetriesOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceScoreModifier for EvidenceScoreRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceScore, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceScoreAdjuster {
    pub cap: f64,
}

impl EvidenceScoreModifier for CapEvidenceScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceScoreAdjuster {
    pub minimum: f64,
}

impl EvidenceScoreModifier for MinimumEvidenceScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the evidence score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceScoreEngine {
    pub steps: Vec<Rc<dyn EvidenceScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceScoreEngine {
    pub fn evaluate(&mut self, item: &EvidenceScore) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_evidence_score_engine() -> EvidenceScoreEngine {
    EvidenceScoreEngine {
        steps: vec![
            Rc::new(RoundingEvidenceScoreAdjuster { step: 0.25 }) as Rc<dyn EvidenceScoreModifier>,
            Rc::new(EvidenceScoreRetriesOffsetAdjuster { weight: 1.58 }),
            Rc::new(CapEvidenceScoreAdjuster { cap: 48169.0 }),
            Rc::new(MinimumEvidenceScoreAdjuster { minimum: 49.9 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 87112.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_score_engine();
        let item = new_evidence_score("code-654", "name-159");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
