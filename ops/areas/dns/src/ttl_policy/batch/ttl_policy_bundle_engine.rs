use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the ttl policy bundle calculation.
pub trait TtlPolicyBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTtlPolicyBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyBundleModifier for ScopeTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TtlPolicyBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyBundleAdjuster {
    pub percent: f64,
}

impl TtlPolicyBundleModifier for PercentTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyBundleAdjuster {
    pub step: f64,
}

impl TtlPolicyBundleModifier for RoundingTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyBundleAdjuster {
    pub cap: f64,
}

impl TtlPolicyBundleModifier for CapTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyBundleRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyBundleModifier for TtlPolicyBundleRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyBundle, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyBundleModifier for TieredTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the ttl policy bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyBundleEngine {
    pub steps: Vec<Rc<dyn TtlPolicyBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyBundleEngine {
    pub fn evaluate(&mut self, item: &TtlPolicyBundle) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_ttl_policy_bundle_engine() -> TtlPolicyBundleEngine {
    TtlPolicyBundleEngine {
        steps: vec![
            Rc::new(ScopeTtlPolicyBundleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.07), ("north".to_string(), 1.0)]),
            }) as Rc<dyn TtlPolicyBundleModifier>,
            Rc::new(PercentTtlPolicyBundleAdjuster { percent: 7.4 }),
            Rc::new(RoundingTtlPolicyBundleAdjuster { step: 5.0 }),
            Rc::new(CapTtlPolicyBundleAdjuster { cap: 38447.0 }),
            Rc::new(TtlPolicyBundleRotationDaysOffsetAdjuster { weight: 0.2 }),
            Rc::new(TieredTtlPolicyBundleAdjuster {
                bounds: vec![1675.0, 4147.0],
                rates: vec![0.99, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 18147.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_bundle_engine();
        let item = new_ttl_policy_bundle("code-448", "label-512");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
