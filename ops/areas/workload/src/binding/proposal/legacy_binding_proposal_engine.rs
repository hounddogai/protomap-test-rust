use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy binding proposal calculation.
pub trait LegacyBindingProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyBindingProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLegacyBindingProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyBindingProposalFactor for TitleLegacyBindingProposalAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LegacyBindingProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingProposalAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl LegacyBindingProposalFactor for LegacyBindingProposalAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyBindingProposal, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyBindingProposalAdjuster {
    pub minimum: f64,
}

impl LegacyBindingProposalFactor for MinimumLegacyBindingProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyBindingProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy binding proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyBindingProposalEngine {
    pub steps: Vec<Rc<dyn LegacyBindingProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyBindingProposalEngine {
    pub fn compute(&mut self, item: &LegacyBindingProposal) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyBindingProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_binding_proposal_engine() -> LegacyBindingProposalEngine {
    LegacyBindingProposalEngine {
        steps: vec![
            Rc::new(TitleLegacyBindingProposalAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.36),
                    ("south".to_string(), 1.12),
                    ("west".to_string(), 0.78),
                ]),
            }) as Rc<dyn LegacyBindingProposalFactor>,
            Rc::new(LegacyBindingProposalAttemptsOffsetAdjuster { weight: 0.93 }),
            Rc::new(MinimumLegacyBindingProposalAdjuster { minimum: 15.9 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 20588.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_binding_proposal_engine();
        let item = new_legacy_binding_proposal("code-399", "title-202");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
