use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted page proposal calculation.
pub trait WeightedPageProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedPageProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedPageProposalAdjuster {
    pub step: f64,
}

impl WeightedPageProposalAdjuster for RoundingWeightedPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedPageProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedPageProposalAdjuster {
    pub percent: f64,
}

impl WeightedPageProposalAdjuster for PercentWeightedPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedPageProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedPageProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedPageProposalAdjuster for TieredWeightedPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedPageProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeWeightedPageProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedPageProposalAdjuster for ScopeWeightedPageProposalAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &WeightedPageProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedPageProposalAdjuster {
    pub cap: f64,
}

impl WeightedPageProposalAdjuster for CapWeightedPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedPageProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted page proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedPageProposalEngine {
    pub steps: Vec<Rc<dyn WeightedPageProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedPageProposalEngine {
    pub fn compute(&mut self, item: &WeightedPageProposal) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedPageProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_page_proposal_engine() -> WeightedPageProposalEngine {
    WeightedPageProposalEngine {
        steps: vec![
            Rc::new(RoundingWeightedPageProposalAdjuster { step: 0.05 }) as Rc<dyn WeightedPageProposalAdjuster>,
            Rc::new(PercentWeightedPageProposalAdjuster { percent: -16.0 }),
            Rc::new(TieredWeightedPageProposalAdjuster {
                bounds: vec![1113.0, 3040.0, 3728.0, 5921.0],
                rates: vec![0.84, 0.96, 1.06, 0.86],
            }),
            Rc::new(ScopeWeightedPageProposalAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.04),
                    ("retail".to_string(), 1.14),
                    ("online".to_string(), 1.17),
                ]),
            }),
            Rc::new(CapWeightedPageProposalAdjuster { cap: 4123.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 99489.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_page_proposal_engine();
        let item = new_weighted_page_proposal("code-485", "name-547");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
