use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual ttl policy override calculation.
pub trait ManualTtlPolicyOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualTtlPolicyOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyOverrideExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl ManualTtlPolicyOverrideModifier for ManualTtlPolicyOverrideExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &ManualTtlPolicyOverride, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualTtlPolicyOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualTtlPolicyOverrideModifier for TieredManualTtlPolicyOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualTtlPolicyOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualTtlPolicyOverrideAdjuster {
    pub step: f64,
}

impl ManualTtlPolicyOverrideModifier for RoundingManualTtlPolicyOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualTtlPolicyOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualTtlPolicyOverrideAdjuster {
    pub percent: f64,
}

impl ManualTtlPolicyOverrideModifier for PercentManualTtlPolicyOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualTtlPolicyOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualTtlPolicyOverrideAdjuster {
    pub cap: f64,
}

impl ManualTtlPolicyOverrideModifier for CapManualTtlPolicyOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualTtlPolicyOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualTtlPolicyOverrideAdjuster {
    pub minimum: f64,
}

impl ManualTtlPolicyOverrideModifier for MinimumManualTtlPolicyOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualTtlPolicyOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual ttl policy override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualTtlPolicyOverrideEngine {
    pub steps: Vec<Rc<dyn ManualTtlPolicyOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualTtlPolicyOverrideEngine {
    pub fn price(&mut self, item: &ManualTtlPolicyOverride) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[ManualTtlPolicyOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_ttl_policy_override_engine() -> ManualTtlPolicyOverrideEngine {
    ManualTtlPolicyOverrideEngine {
        steps: vec![
            Rc::new(ManualTtlPolicyOverrideExpiresAtOffsetAdjuster { weight: 1.82 })
                as Rc<dyn ManualTtlPolicyOverrideModifier>,
            Rc::new(TieredManualTtlPolicyOverrideAdjuster {
                bounds: vec![52.0, 2542.0, 4243.0, 8506.0],
                rates: vec![0.81, 1.17, 0.82, 0.88],
            }),
            Rc::new(RoundingManualTtlPolicyOverrideAdjuster { step: 0.25 }),
            Rc::new(PercentManualTtlPolicyOverrideAdjuster { percent: -22.9 }),
            Rc::new(CapManualTtlPolicyOverrideAdjuster { cap: 12534.0 }),
            Rc::new(MinimumManualTtlPolicyOverrideAdjuster { minimum: 30.2 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 13629.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_ttl_policy_override_engine();
        let item = new_manual_ttl_policy_override("code-118", "name-982");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
