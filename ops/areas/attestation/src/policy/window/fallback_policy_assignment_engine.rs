use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback policy assignment calculation.
pub trait FallbackPolicyAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPolicyAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPolicyAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPolicyAssignmentStep for TieredFallbackPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPolicyAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackPolicyAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackPolicyAssignmentStep for FlaggedFallbackPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackPolicyAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPolicyAssignmentAdjuster {
    pub minimum: f64,
}

impl FallbackPolicyAssignmentStep for MinimumFallbackPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPolicyAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackPolicyAssignmentAdjuster {
    pub cap: f64,
}

impl FallbackPolicyAssignmentStep for CapFallbackPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackPolicyAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackPolicyAssignmentAdjuster {
    pub step: f64,
}

impl FallbackPolicyAssignmentStep for RoundingFallbackPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackPolicyAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackPolicyAssignmentAdjuster {
    pub percent: f64,
}

impl FallbackPolicyAssignmentStep for PercentFallbackPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackPolicyAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback policy assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPolicyAssignmentEngine {
    pub steps: Vec<Rc<dyn FallbackPolicyAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPolicyAssignmentEngine {
    pub fn evaluate(&mut self, item: &FallbackPolicyAssignment) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackPolicyAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPolicyAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_policy_assignment_engine(flags: &flags::Registry) -> FallbackPolicyAssignmentEngine {
    FallbackPolicyAssignmentEngine {
        steps: vec![
            Rc::new(TieredFallbackPolicyAssignmentAdjuster {
                bounds: vec![5739.0, 7720.0, 7870.0, 7885.0],
                rates: vec![0.91, 1.16, 1.09, 1.12],
            }) as Rc<dyn FallbackPolicyAssignmentStep>,
            Rc::new(FlaggedFallbackPolicyAssignmentAdjuster {
                flags: flags.clone(),
                flag: "fallback-policy-assignment-tiered".to_string(),
                percent: -5.9,
            }),
            Rc::new(MinimumFallbackPolicyAssignmentAdjuster { minimum: 36.9 }),
            Rc::new(CapFallbackPolicyAssignmentAdjuster { cap: 34596.0 }),
            Rc::new(RoundingFallbackPolicyAssignmentAdjuster { step: 5.0 }),
            Rc::new(PercentFallbackPolicyAssignmentAdjuster { percent: 6.2 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 8173.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_policy_assignment_engine(&flags::new_registry());
        let item = new_fallback_policy_assignment("code-490", "label-178");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
