use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient flow log proposal calculation.
pub trait LenientFlowLogProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientFlowLogProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientFlowLogProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientFlowLogProposalModifier for TieredLenientFlowLogProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientFlowLogProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLenientFlowLogProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientFlowLogProposalModifier for UnitLenientFlowLogProposalAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LenientFlowLogProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientFlowLogProposalAdjuster {
    pub cap: f64,
}

impl LenientFlowLogProposalModifier for CapLenientFlowLogProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientFlowLogProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientFlowLogProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientFlowLogProposalModifier for FlaggedLenientFlowLogProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientFlowLogProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientFlowLogProposalAdjuster {
    pub step: f64,
}

impl LenientFlowLogProposalModifier for RoundingLenientFlowLogProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientFlowLogProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogProposalCapacityOffsetAdjuster {
    pub weight: f64,
}

impl LenientFlowLogProposalModifier for LenientFlowLogProposalCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &LenientFlowLogProposal, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the lenient flow log proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientFlowLogProposalEngine {
    pub steps: Vec<Rc<dyn LenientFlowLogProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientFlowLogProposalEngine {
    pub fn evaluate(&mut self, item: &LenientFlowLogProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientFlowLogProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientFlowLogProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_flow_log_proposal_engine(flags: &flags::Registry) -> LenientFlowLogProposalEngine {
    LenientFlowLogProposalEngine {
        steps: vec![
            Rc::new(TieredLenientFlowLogProposalAdjuster {
                bounds: vec![4640.0, 5518.0, 6319.0, 7194.0],
                rates: vec![0.89, 1.1, 1.04, 1.1],
            }) as Rc<dyn LenientFlowLogProposalModifier>,
            Rc::new(UnitLenientFlowLogProposalAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.75),
                    ("central".to_string(), 0.91),
                    ("south".to_string(), 0.86),
                ]),
            }),
            Rc::new(CapLenientFlowLogProposalAdjuster { cap: 37364.0 }),
            Rc::new(FlaggedLenientFlowLogProposalAdjuster {
                flags: flags.clone(),
                flag: "lenient-flow-log-proposal-seasonal".to_string(),
                percent: 4.1,
            }),
            Rc::new(RoundingLenientFlowLogProposalAdjuster { step: 0.1 }),
            Rc::new(LenientFlowLogProposalCapacityOffsetAdjuster { weight: 1.94 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 2824.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_flow_log_proposal_engine(&flags::new_registry());
        let item = new_lenient_flow_log_proposal("code-426", "label-967");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
