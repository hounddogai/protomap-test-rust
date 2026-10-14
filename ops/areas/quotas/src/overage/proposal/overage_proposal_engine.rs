use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the overage proposal calculation.
pub trait OverageProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageProposalAdjuster {
    pub cap: f64,
}

impl OverageProposalAdjuster for CapOverageProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageProposalWeightOffsetAdjuster {
    pub weight: f64,
}

impl OverageProposalAdjuster for OverageProposalWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &OverageProposal, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageProposalAdjuster for TieredOverageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelOverageProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OverageProposalAdjuster for LabelOverageProposalAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &OverageProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOverageProposalAdjuster {
    pub percent: f64,
}

impl OverageProposalAdjuster for PercentOverageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OverageProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageProposalAdjuster {
    pub step: f64,
}

impl OverageProposalAdjuster for RoundingOverageProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the overage proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageProposalEngine {
    pub steps: Vec<Rc<dyn OverageProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageProposalEngine {
    pub fn evaluate(&mut self, item: &OverageProposal) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageProposalAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_overage_proposal_engine() -> OverageProposalEngine {
    OverageProposalEngine {
        steps: vec![
            Rc::new(CapOverageProposalAdjuster { cap: 1535.0 }) as Rc<dyn OverageProposalAdjuster>,
            Rc::new(OverageProposalWeightOffsetAdjuster { weight: 0.47 }),
            Rc::new(TieredOverageProposalAdjuster {
                bounds: vec![36.0, 1993.0, 2011.0, 4054.0],
                rates: vec![0.91, 0.98, 1.03, 0.86],
            }),
            Rc::new(LabelOverageProposalAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.14), ("east".to_string(), 0.77)]),
            }),
            Rc::new(PercentOverageProposalAdjuster { percent: -14.5 }),
            Rc::new(RoundingOverageProposalAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 88580.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_proposal_engine();
        let item = new_overage_proposal("code-131", "label-193");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
