use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered usage proposal calculation.
pub trait TieredUsageProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredUsageProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredUsageProposalAdjuster {
    pub step: f64,
}

impl TieredUsageProposalFactor for RoundingTieredUsageProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredUsageProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryTieredUsageProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredUsageProposalFactor for CategoryTieredUsageProposalAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &TieredUsageProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredUsageProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredUsageProposalFactor for TieredTieredUsageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredUsageProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageProposalAmountOffsetAdjuster {
    pub weight: f64,
}

impl TieredUsageProposalFactor for TieredUsageProposalAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TieredUsageProposal, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the tiered usage proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredUsageProposalEngine {
    pub steps: Vec<Rc<dyn TieredUsageProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredUsageProposalEngine {
    pub fn run(&mut self, item: &TieredUsageProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredUsageProposalFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredUsageProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_usage_proposal_engine() -> TieredUsageProposalEngine {
    TieredUsageProposalEngine {
        steps: vec![
            Rc::new(RoundingTieredUsageProposalAdjuster { step: 5.0 }) as Rc<dyn TieredUsageProposalFactor>,
            Rc::new(CategoryTieredUsageProposalAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.4), ("south".to_string(), 0.7)]),
            }),
            Rc::new(TieredTieredUsageProposalAdjuster {
                bounds: vec![4304.0, 8220.0],
                rates: vec![1.09, 1.13],
            }),
            Rc::new(TieredUsageProposalAmountOffsetAdjuster { weight: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 83160.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_usage_proposal_engine();
        let item = new_tiered_usage_proposal("code-137", "label-439");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
