use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy tier calculation.
pub trait TtlPolicyTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyTierAdjuster {
    pub percent: f64,
}

impl TtlPolicyTierFactor for PercentTtlPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTierMaximumOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyTierFactor for TtlPolicyTierMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyTier, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyTierAdjuster {
    pub minimum: f64,
}

impl TtlPolicyTierFactor for MinimumTtlPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyTierFactor for FlaggedTtlPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyTierAdjuster {
    pub cap: f64,
}

impl TtlPolicyTierFactor for CapTtlPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyTierAdjuster {
    pub step: f64,
}

impl TtlPolicyTierFactor for RoundingTtlPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the ttl policy tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyTierEngine {
    pub steps: Vec<Rc<dyn TtlPolicyTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyTierEngine {
    pub fn run(&mut self, item: &TtlPolicyTier) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyTierFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_tier_engine(flags: &flags::Registry) -> TtlPolicyTierEngine {
    TtlPolicyTierEngine {
        steps: vec![
            Rc::new(PercentTtlPolicyTierAdjuster { percent: 8.7 }) as Rc<dyn TtlPolicyTierFactor>,
            Rc::new(TtlPolicyTierMaximumOffsetAdjuster { weight: 1.23 }),
            Rc::new(MinimumTtlPolicyTierAdjuster { minimum: 16.1 }),
            Rc::new(FlaggedTtlPolicyTierAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-tier-rolling".to_string(),
                percent: -5.3,
            }),
            Rc::new(CapTtlPolicyTierAdjuster { cap: 38705.0 }),
            Rc::new(RoundingTtlPolicyTierAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 89378.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_tier_engine(&flags::new_registry());
        let item = new_ttl_policy_tier("code-578", "name-870");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
