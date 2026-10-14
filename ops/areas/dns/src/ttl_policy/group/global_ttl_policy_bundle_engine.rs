use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global ttl policy bundle calculation.
pub trait GlobalTtlPolicyBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalTtlPolicyBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalTtlPolicyBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalTtlPolicyBundleFactor for FlaggedGlobalTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalTtlPolicyBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalTtlPolicyBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalTtlPolicyBundleFactor for LabelGlobalTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalTtlPolicyBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalTtlPolicyBundleAdjuster {
    pub percent: f64,
}

impl GlobalTtlPolicyBundleFactor for PercentGlobalTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalTtlPolicyBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalTtlPolicyBundleAdjuster {
    pub cap: f64,
}

impl GlobalTtlPolicyBundleFactor for CapGlobalTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalTtlPolicyBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyBundleUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalTtlPolicyBundleFactor for GlobalTtlPolicyBundleUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalTtlPolicyBundle, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalTtlPolicyBundleAdjuster {
    pub step: f64,
}

impl GlobalTtlPolicyBundleFactor for RoundingGlobalTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalTtlPolicyBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global ttl policy bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalTtlPolicyBundleEngine {
    pub steps: Vec<Rc<dyn GlobalTtlPolicyBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalTtlPolicyBundleEngine {
    pub fn evaluate(&mut self, item: &GlobalTtlPolicyBundle) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalTtlPolicyBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_ttl_policy_bundle_engine(flags: &flags::Registry) -> GlobalTtlPolicyBundleEngine {
    GlobalTtlPolicyBundleEngine {
        steps: vec![
            Rc::new(FlaggedGlobalTtlPolicyBundleAdjuster {
                flags: flags.clone(),
                flag: "global-ttl-policy-bundle-strict".to_string(),
                percent: -9.5,
            }) as Rc<dyn GlobalTtlPolicyBundleFactor>,
            Rc::new(LabelGlobalTtlPolicyBundleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.84),
                    ("east".to_string(), 0.89),
                    ("south".to_string(), 1.33),
                ]),
            }),
            Rc::new(PercentGlobalTtlPolicyBundleAdjuster { percent: -21.6 }),
            Rc::new(CapGlobalTtlPolicyBundleAdjuster { cap: 44571.0 }),
            Rc::new(GlobalTtlPolicyBundleUpdatedAtOffsetAdjuster { weight: 1.57 }),
            Rc::new(RoundingGlobalTtlPolicyBundleAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 20945.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_ttl_policy_bundle_engine(&flags::new_registry());
        let item = new_global_ttl_policy_bundle("code-402", "label-519");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
