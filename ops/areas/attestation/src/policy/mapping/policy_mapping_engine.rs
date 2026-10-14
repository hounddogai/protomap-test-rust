use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy mapping calculation.
pub trait PolicyMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMappingQuantityOffsetAdjuster {
    pub weight: f64,
}

impl PolicyMappingStep for PolicyMappingQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyMapping, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyMappingAdjuster {
    pub cap: f64,
}

impl PolicyMappingStep for CapPolicyMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyMappingStep for TieredPolicyMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyMappingAdjuster {
    pub step: f64,
}

impl PolicyMappingStep for RoundingPolicyMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyMappingStep for FlaggedPolicyMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyMappingAdjuster {
    pub minimum: f64,
}

impl PolicyMappingStep for MinimumPolicyMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the policy mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyMappingEngine {
    pub steps: Vec<Rc<dyn PolicyMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyMappingEngine {
    pub fn evaluate(&mut self, item: &PolicyMapping) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyMappingStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_policy_mapping_engine(flags: &flags::Registry) -> PolicyMappingEngine {
    PolicyMappingEngine {
        steps: vec![
            Rc::new(PolicyMappingQuantityOffsetAdjuster { weight: 1.3 }) as Rc<dyn PolicyMappingStep>,
            Rc::new(CapPolicyMappingAdjuster { cap: 4821.0 }),
            Rc::new(TieredPolicyMappingAdjuster {
                bounds: vec![3287.0, 6314.0],
                rates: vec![1.13, 0.81],
            }),
            Rc::new(RoundingPolicyMappingAdjuster { step: 0.25 }),
            Rc::new(FlaggedPolicyMappingAdjuster {
                flags: flags.clone(),
                flag: "policy-mapping-default".to_string(),
                percent: -7.9,
            }),
            Rc::new(MinimumPolicyMappingAdjuster { minimum: 26.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 96422.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_mapping_engine(&flags::new_registry());
        let item = new_policy_mapping("code-857", "label-870");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
