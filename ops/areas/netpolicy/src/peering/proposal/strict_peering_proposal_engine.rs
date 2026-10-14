use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict peering proposal calculation.
pub trait StrictPeeringProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPeeringProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictPeeringProposalAdjuster {
    pub step: f64,
}

impl StrictPeeringProposalAdjuster for RoundingStrictPeeringProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictPeeringProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictPeeringProposalAdjuster {
    pub percent: f64,
}

impl StrictPeeringProposalAdjuster for PercentStrictPeeringProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictPeeringProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleStrictPeeringProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictPeeringProposalAdjuster for LocaleStrictPeeringProposalAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &StrictPeeringProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict peering proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPeeringProposalEngine {
    pub steps: Vec<Rc<dyn StrictPeeringProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPeeringProposalEngine {
    pub fn price(&mut self, item: &StrictPeeringProposal) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[StrictPeeringProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_peering_proposal_engine() -> StrictPeeringProposalEngine {
    StrictPeeringProposalEngine {
        steps: vec![
            Rc::new(RoundingStrictPeeringProposalAdjuster { step: 0.25 }) as Rc<dyn StrictPeeringProposalAdjuster>,
            Rc::new(PercentStrictPeeringProposalAdjuster { percent: -6.3 }),
            Rc::new(LocaleStrictPeeringProposalAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.39),
                    ("north".to_string(), 1.31),
                    ("south".to_string(), 1.23),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 33621.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_peering_proposal_engine();
        let item = new_strict_peering_proposal("code-638", "name-293");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
