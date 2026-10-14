use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary plan proposal calculation.
pub trait PrimaryPlanProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryPlanProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryPlanProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryPlanProposalAdjuster for TieredPrimaryPlanProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryPlanProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryPlanProposalAdjuster {
    pub step: f64,
}

impl PrimaryPlanProposalAdjuster for RoundingPrimaryPlanProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryPlanProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryPlanProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryPlanProposalAdjuster for TitlePrimaryPlanProposalAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryPlanProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryPlanProposalAdjuster {
    pub percent: f64,
}

impl PrimaryPlanProposalAdjuster for PercentPrimaryPlanProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryPlanProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryPlanProposalAdjuster {
    pub minimum: f64,
}

impl PrimaryPlanProposalAdjuster for MinimumPrimaryPlanProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryPlanProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryPlanProposalAdjuster {
    pub cap: f64,
}

impl PrimaryPlanProposalAdjuster for CapPrimaryPlanProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryPlanProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary plan proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryPlanProposalEngine {
    pub steps: Vec<Rc<dyn PrimaryPlanProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryPlanProposalEngine {
    pub fn price(&mut self, item: &PrimaryPlanProposal) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryPlanProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_plan_proposal_engine() -> PrimaryPlanProposalEngine {
    PrimaryPlanProposalEngine {
        steps: vec![
            Rc::new(TieredPrimaryPlanProposalAdjuster {
                bounds: vec![121.0, 605.0],
                rates: vec![1.16, 1.13],
            }) as Rc<dyn PrimaryPlanProposalAdjuster>,
            Rc::new(RoundingPrimaryPlanProposalAdjuster { step: 0.5 }),
            Rc::new(TitlePrimaryPlanProposalAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.0)]),
            }),
            Rc::new(PercentPrimaryPlanProposalAdjuster { percent: 8.9 }),
            Rc::new(MinimumPrimaryPlanProposalAdjuster { minimum: 11.8 }),
            Rc::new(CapPrimaryPlanProposalAdjuster { cap: 23572.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 17125.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_plan_proposal_engine();
        let item = new_primary_plan_proposal("code-486", "title-116");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
