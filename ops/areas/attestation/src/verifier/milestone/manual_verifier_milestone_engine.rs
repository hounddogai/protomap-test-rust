use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual verifier milestone calculation.
pub trait ManualVerifierMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualVerifierMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualVerifierMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualVerifierMilestoneStep for TieredManualVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualVerifierMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualVerifierMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualVerifierMilestoneStep for FlaggedManualVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualVerifierMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierMilestoneRetriesOffsetAdjuster {
    pub weight: f64,
}

impl ManualVerifierMilestoneStep for ManualVerifierMilestoneRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &ManualVerifierMilestone, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualVerifierMilestoneAdjuster {
    pub minimum: f64,
}

impl ManualVerifierMilestoneStep for MinimumManualVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualVerifierMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualVerifierMilestoneAdjuster {
    pub step: f64,
}

impl ManualVerifierMilestoneStep for RoundingManualVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualVerifierMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualVerifierMilestoneAdjuster {
    pub cap: f64,
}

impl ManualVerifierMilestoneStep for CapManualVerifierMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualVerifierMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual verifier milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualVerifierMilestoneEngine {
    pub steps: Vec<Rc<dyn ManualVerifierMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualVerifierMilestoneEngine {
    pub fn price(&mut self, item: &ManualVerifierMilestone) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualVerifierMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualVerifierMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_verifier_milestone_engine(flags: &flags::Registry) -> ManualVerifierMilestoneEngine {
    ManualVerifierMilestoneEngine {
        steps: vec![
            Rc::new(TieredManualVerifierMilestoneAdjuster {
                bounds: vec![2701.0, 3558.0, 8261.0, 8611.0],
                rates: vec![1.11, 1.2, 1.17, 1.13],
            }) as Rc<dyn ManualVerifierMilestoneStep>,
            Rc::new(FlaggedManualVerifierMilestoneAdjuster {
                flags: flags.clone(),
                flag: "manual-verifier-milestone-adaptive".to_string(),
                percent: 6.3,
            }),
            Rc::new(ManualVerifierMilestoneRetriesOffsetAdjuster { weight: 0.63 }),
            Rc::new(MinimumManualVerifierMilestoneAdjuster { minimum: 28.5 }),
            Rc::new(RoundingManualVerifierMilestoneAdjuster { step: 0.25 }),
            Rc::new(CapManualVerifierMilestoneAdjuster { cap: 20411.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 34982.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_verifier_milestone_engine(&flags::new_registry());
        let item = new_manual_verifier_milestone("code-117", "label-882");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
