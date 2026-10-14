use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual ttl policy threshold calculation.
pub trait ManualTtlPolicyThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualTtlPolicyThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualTtlPolicyThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualTtlPolicyThresholdAdjuster for FlaggedManualTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualTtlPolicyThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualTtlPolicyThresholdAdjuster {
    pub step: f64,
}

impl ManualTtlPolicyThresholdAdjuster for RoundingManualTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualTtlPolicyThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyThresholdQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ManualTtlPolicyThresholdAdjuster for ManualTtlPolicyThresholdQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ManualTtlPolicyThreshold, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualTtlPolicyThresholdAdjuster {
    pub cap: f64,
}

impl ManualTtlPolicyThresholdAdjuster for CapManualTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualTtlPolicyThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualTtlPolicyThresholdAdjuster {
    pub minimum: f64,
}

impl ManualTtlPolicyThresholdAdjuster for MinimumManualTtlPolicyThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualTtlPolicyThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual ttl policy threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualTtlPolicyThresholdEngine {
    pub steps: Vec<Rc<dyn ManualTtlPolicyThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualTtlPolicyThresholdEngine {
    pub fn compute(&mut self, item: &ManualTtlPolicyThreshold) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualTtlPolicyThresholdAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualTtlPolicyThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_ttl_policy_threshold_engine(flags: &flags::Registry) -> ManualTtlPolicyThresholdEngine {
    ManualTtlPolicyThresholdEngine {
        steps: vec![
            Rc::new(FlaggedManualTtlPolicyThresholdAdjuster {
                flags: flags.clone(),
                flag: "manual-ttl-policy-threshold-fallback".to_string(),
                percent: 3.7,
            }) as Rc<dyn ManualTtlPolicyThresholdAdjuster>,
            Rc::new(RoundingManualTtlPolicyThresholdAdjuster { step: 0.05 }),
            Rc::new(ManualTtlPolicyThresholdQuantityOffsetAdjuster { weight: 0.62 }),
            Rc::new(CapManualTtlPolicyThresholdAdjuster { cap: 20753.0 }),
            Rc::new(MinimumManualTtlPolicyThresholdAdjuster { minimum: 43.6 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 10940.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_ttl_policy_threshold_engine(&flags::new_registry());
        let item = new_manual_ttl_policy_threshold("code-942", "title-24");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
