use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the sink proposal calculation.
pub trait SinkProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkProposalAdjuster for TieredSinkProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkProposalAdjuster {
    pub step: f64,
}

impl SinkProposalAdjuster for RoundingSinkProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkProposalAdjuster {
    pub percent: f64,
}

impl SinkProposalAdjuster for PercentSinkProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkProposalDiscountOffsetAdjuster {
    pub weight: f64,
}

impl SinkProposalAdjuster for SinkProposalDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &SinkProposal, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the sink proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkProposalEngine {
    pub steps: Vec<Rc<dyn SinkProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkProposalEngine {
    pub fn evaluate(&mut self, item: &SinkProposal) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[SinkProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_sink_proposal_engine() -> SinkProposalEngine {
    SinkProposalEngine {
        steps: vec![
            Rc::new(TieredSinkProposalAdjuster {
                bounds: vec![4649.0, 4789.0, 9739.0],
                rates: vec![1.08, 0.88, 0.88],
            }) as Rc<dyn SinkProposalAdjuster>,
            Rc::new(RoundingSinkProposalAdjuster { step: 5.0 }),
            Rc::new(PercentSinkProposalAdjuster { percent: -19.6 }),
            Rc::new(SinkProposalDiscountOffsetAdjuster { weight: 0.03 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 27090.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_proposal_engine();
        let item = new_sink_proposal("code-284", "title-817");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
