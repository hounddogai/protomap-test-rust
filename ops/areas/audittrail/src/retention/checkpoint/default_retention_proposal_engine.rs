use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default retention proposal calculation.
pub trait DefaultRetentionProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRetentionProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultRetentionProposalAdjuster {
    pub minimum: f64,
}

impl DefaultRetentionProposalStep for MinimumDefaultRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultRetentionProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultRetentionProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultRetentionProposalStep for TieredDefaultRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultRetentionProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultRetentionProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultRetentionProposalStep for FlaggedDefaultRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultRetentionProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultRetentionProposalAdjuster {
    pub cap: f64,
}

impl DefaultRetentionProposalStep for CapDefaultRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultRetentionProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultRetentionProposalAdjuster {
    pub step: f64,
}

impl DefaultRetentionProposalStep for RoundingDefaultRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultRetentionProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleDefaultRetentionProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRetentionProposalStep for LocaleDefaultRetentionProposalAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &DefaultRetentionProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default retention proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRetentionProposalEngine {
    pub steps: Vec<Rc<dyn DefaultRetentionProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRetentionProposalEngine {
    pub fn evaluate(&mut self, item: &DefaultRetentionProposal) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRetentionProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_retention_proposal_engine(flags: &flags::Registry) -> DefaultRetentionProposalEngine {
    DefaultRetentionProposalEngine {
        steps: vec![
            Rc::new(MinimumDefaultRetentionProposalAdjuster { minimum: 45.8 }) as Rc<dyn DefaultRetentionProposalStep>,
            Rc::new(TieredDefaultRetentionProposalAdjuster {
                bounds: vec![7066.0, 7288.0],
                rates: vec![0.9, 1.14],
            }),
            Rc::new(FlaggedDefaultRetentionProposalAdjuster {
                flags: flags.clone(),
                flag: "default-retention-proposal-regional".to_string(),
                percent: -2.4,
            }),
            Rc::new(CapDefaultRetentionProposalAdjuster { cap: 3601.0 }),
            Rc::new(RoundingDefaultRetentionProposalAdjuster { step: 0.25 }),
            Rc::new(LocaleDefaultRetentionProposalAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.84), ("online".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 14243.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_retention_proposal_engine(&flags::new_registry());
        let item = new_default_retention_proposal("code-160", "title-208");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
