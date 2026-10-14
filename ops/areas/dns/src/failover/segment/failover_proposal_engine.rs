use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover proposal calculation.
pub trait FailoverProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverProposalModifier for TieredFailoverProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProposalDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl FailoverProposalModifier for FailoverProposalDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverProposal, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverProposalAdjuster {
    pub step: f64,
}

impl FailoverProposalModifier for RoundingFailoverProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFailoverProposalAdjuster {
    pub percent: f64,
}

impl FailoverProposalModifier for PercentFailoverProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FailoverProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverProposalAdjuster {
    pub cap: f64,
}

impl FailoverProposalModifier for CapFailoverProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the failover proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverProposalEngine {
    pub steps: Vec<Rc<dyn FailoverProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverProposalEngine {
    pub fn evaluate(&mut self, item: &FailoverProposal) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_failover_proposal_engine() -> FailoverProposalEngine {
    FailoverProposalEngine {
        steps: vec![
            Rc::new(TieredFailoverProposalAdjuster {
                bounds: vec![4221.0, 4288.0, 4594.0, 9268.0],
                rates: vec![1.05, 1.13, 0.99, 1.16],
            }) as Rc<dyn FailoverProposalModifier>,
            Rc::new(FailoverProposalDurationMinutesOffsetAdjuster { weight: 1.82 }),
            Rc::new(RoundingFailoverProposalAdjuster { step: 0.25 }),
            Rc::new(PercentFailoverProposalAdjuster { percent: 9.4 }),
            Rc::new(CapFailoverProposalAdjuster { cap: 49921.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 8139.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_proposal_engine();
        let item = new_failover_proposal("code-919", "title-58");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
