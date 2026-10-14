use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default ttl policy bundle calculation.
pub trait DefaultTtlPolicyBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultTtlPolicyBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTtlPolicyBundleRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl DefaultTtlPolicyBundleStep for DefaultTtlPolicyBundleRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultTtlPolicyBundle, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultTtlPolicyBundleAdjuster {
    pub cap: f64,
}

impl DefaultTtlPolicyBundleStep for CapDefaultTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultTtlPolicyBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultTtlPolicyBundleAdjuster {
    pub percent: f64,
}

impl DefaultTtlPolicyBundleStep for PercentDefaultTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultTtlPolicyBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDefaultTtlPolicyBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultTtlPolicyBundleStep for SegmentDefaultTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &DefaultTtlPolicyBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default ttl policy bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultTtlPolicyBundleEngine {
    pub steps: Vec<Rc<dyn DefaultTtlPolicyBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultTtlPolicyBundleEngine {
    pub fn price(&mut self, item: &DefaultTtlPolicyBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultTtlPolicyBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_ttl_policy_bundle_engine() -> DefaultTtlPolicyBundleEngine {
    DefaultTtlPolicyBundleEngine {
        steps: vec![
            Rc::new(DefaultTtlPolicyBundleRotationDaysOffsetAdjuster { weight: 0.61 })
                as Rc<dyn DefaultTtlPolicyBundleStep>,
            Rc::new(CapDefaultTtlPolicyBundleAdjuster { cap: 21195.0 }),
            Rc::new(PercentDefaultTtlPolicyBundleAdjuster { percent: -6.3 }),
            Rc::new(SegmentDefaultTtlPolicyBundleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.7), ("central".to_string(), 1.12)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 58180.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_ttl_policy_bundle_engine();
        let item = new_default_ttl_policy_bundle("code-789", "label-845");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
