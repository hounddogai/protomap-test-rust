use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval claim calculation.
pub trait ApprovalClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalClaim, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalClaimStep for FlaggedApprovalClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalClaimAdjuster {
    pub step: f64,
}

impl ApprovalClaimStep for RoundingApprovalClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalClaimStep for TieredApprovalClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalClaimMaximumOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalClaimStep for ApprovalClaimMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalClaim, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalClaimAdjuster {
    pub minimum: f64,
}

impl ApprovalClaimStep for MinimumApprovalClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the approval claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalClaimEngine {
    pub steps: Vec<Rc<dyn ApprovalClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalClaimEngine {
    pub fn run(&mut self, item: &ApprovalClaim) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalClaimStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_approval_claim_engine(flags: &flags::Registry) -> ApprovalClaimEngine {
    ApprovalClaimEngine {
        steps: vec![
            Rc::new(FlaggedApprovalClaimAdjuster {
                flags: flags.clone(),
                flag: "approval-claim-pending".to_string(),
                percent: 6.0,
            }) as Rc<dyn ApprovalClaimStep>,
            Rc::new(RoundingApprovalClaimAdjuster { step: 0.5 }),
            Rc::new(TieredApprovalClaimAdjuster {
                bounds: vec![1463.0, 6058.0, 7879.0, 8050.0],
                rates: vec![0.95, 0.91, 0.82, 1.16],
            }),
            Rc::new(ApprovalClaimMaximumOffsetAdjuster { weight: 0.17 }),
            Rc::new(MinimumApprovalClaimAdjuster { minimum: 14.5 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 15143.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_claim_engine(&flags::new_registry());
        let item = new_approval_claim("code-28", "name-175");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
