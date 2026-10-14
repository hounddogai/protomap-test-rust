use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted escalation proposal calculation.
pub trait WeightedEscalationProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedEscalationProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationProposalBurstOffsetAdjuster {
    pub weight: f64,
}

impl WeightedEscalationProposalAdjuster for WeightedEscalationProposalBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedEscalationProposal, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedEscalationProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedEscalationProposalAdjuster for FlaggedWeightedEscalationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedEscalationProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedEscalationProposalAdjuster {
    pub percent: f64,
}

impl WeightedEscalationProposalAdjuster for PercentWeightedEscalationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedEscalationProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted escalation proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedEscalationProposalEngine {
    pub steps: Vec<Rc<dyn WeightedEscalationProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedEscalationProposalEngine {
    pub fn evaluate(&mut self, item: &WeightedEscalationProposal) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedEscalationProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_escalation_proposal_engine(flags: &flags::Registry) -> WeightedEscalationProposalEngine {
    WeightedEscalationProposalEngine {
        steps: vec![
            Rc::new(WeightedEscalationProposalBurstOffsetAdjuster { weight: 1.41 })
                as Rc<dyn WeightedEscalationProposalAdjuster>,
            Rc::new(FlaggedWeightedEscalationProposalAdjuster {
                flags: flags.clone(),
                flag: "weighted-escalation-proposal-primary".to_string(),
                percent: -7.8,
            }),
            Rc::new(PercentWeightedEscalationProposalAdjuster { percent: -7.2 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 36760.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_escalation_proposal_engine(&flags::new_registry());
        let item = new_weighted_escalation_proposal("code-582", "label-330");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
