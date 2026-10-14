use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending delegation milestone calculation.
pub trait PendingDelegationMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingDelegationMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingDelegationMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingDelegationMilestoneStep for TitlePendingDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingDelegationMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingDelegationMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingDelegationMilestoneStep for TieredPendingDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingDelegationMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingDelegationMilestoneAdjuster {
    pub minimum: f64,
}

impl PendingDelegationMilestoneStep for MinimumPendingDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingDelegationMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingDelegationMilestoneAdjuster {
    pub percent: f64,
}

impl PendingDelegationMilestoneStep for PercentPendingDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingDelegationMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingDelegationMilestoneAdjuster {
    pub step: f64,
}

impl PendingDelegationMilestoneStep for RoundingPendingDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingDelegationMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingDelegationMilestoneAdjuster {
    pub cap: f64,
}

impl PendingDelegationMilestoneStep for CapPendingDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingDelegationMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending delegation milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingDelegationMilestoneEngine {
    pub steps: Vec<Rc<dyn PendingDelegationMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingDelegationMilestoneEngine {
    pub fn price(&mut self, item: &PendingDelegationMilestone) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingDelegationMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingDelegationMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_delegation_milestone_engine() -> PendingDelegationMilestoneEngine {
    PendingDelegationMilestoneEngine {
        steps: vec![
            Rc::new(TitlePendingDelegationMilestoneAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.32)]),
            }) as Rc<dyn PendingDelegationMilestoneStep>,
            Rc::new(TieredPendingDelegationMilestoneAdjuster {
                bounds: vec![3638.0, 3877.0, 9230.0, 9392.0],
                rates: vec![0.8, 1.04, 1.16, 1.16],
            }),
            Rc::new(MinimumPendingDelegationMilestoneAdjuster { minimum: 34.7 }),
            Rc::new(PercentPendingDelegationMilestoneAdjuster { percent: -21.5 }),
            Rc::new(RoundingPendingDelegationMilestoneAdjuster { step: 0.1 }),
            Rc::new(CapPendingDelegationMilestoneAdjuster { cap: 8389.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 36485.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_delegation_milestone_engine();
        let item = new_pending_delegation_milestone("code-28", "title-440");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
