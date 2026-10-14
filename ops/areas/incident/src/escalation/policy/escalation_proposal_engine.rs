use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the escalation proposal calculation.
pub trait EscalationProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeEscalationProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EscalationProposalModifier for ScopeEscalationProposalAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &EscalationProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEscalationProposalAdjuster {
    pub cap: f64,
}

impl EscalationProposalModifier for CapEscalationProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EscalationProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProposalLimitOffsetAdjuster {
    pub weight: f64,
}

impl EscalationProposalModifier for EscalationProposalLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationProposal, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationProposalModifier for TieredEscalationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the escalation proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationProposalEngine {
    pub steps: Vec<Rc<dyn EscalationProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationProposalEngine {
    pub fn run(&mut self, item: &EscalationProposal) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EscalationProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EscalationProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_escalation_proposal_engine() -> EscalationProposalEngine {
    EscalationProposalEngine {
        steps: vec![
            Rc::new(ScopeEscalationProposalAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.81), ("south".to_string(), 1.1)]),
            }) as Rc<dyn EscalationProposalModifier>,
            Rc::new(CapEscalationProposalAdjuster { cap: 510.0 }),
            Rc::new(EscalationProposalLimitOffsetAdjuster { weight: 1.65 }),
            Rc::new(TieredEscalationProposalAdjuster {
                bounds: vec![777.0, 1096.0, 6997.0, 9808.0],
                rates: vec![0.95, 0.86, 1.18, 1.2],
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 74268.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_proposal_engine();
        let item = new_escalation_proposal("code-667", "title-232");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
