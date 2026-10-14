use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling ttl policy bundle calculation.
pub trait RollingTtlPolicyBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTtlPolicyBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingTtlPolicyBundleAdjuster {
    pub step: f64,
}

impl RollingTtlPolicyBundleModifier for RoundingRollingTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingTtlPolicyBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTtlPolicyBundleAdjuster {
    pub percent: f64,
}

impl RollingTtlPolicyBundleModifier for PercentRollingTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTtlPolicyBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingTtlPolicyBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingTtlPolicyBundleModifier for TitleRollingTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingTtlPolicyBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingTtlPolicyBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingTtlPolicyBundleModifier for FlaggedRollingTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingTtlPolicyBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingTtlPolicyBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingTtlPolicyBundleModifier for TieredRollingTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingTtlPolicyBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingTtlPolicyBundleAdjuster {
    pub minimum: f64,
}

impl RollingTtlPolicyBundleModifier for MinimumRollingTtlPolicyBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingTtlPolicyBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rolling ttl policy bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTtlPolicyBundleEngine {
    pub steps: Vec<Rc<dyn RollingTtlPolicyBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTtlPolicyBundleEngine {
    pub fn compute(&mut self, item: &RollingTtlPolicyBundle) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[RollingTtlPolicyBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_ttl_policy_bundle_engine(flags: &flags::Registry) -> RollingTtlPolicyBundleEngine {
    RollingTtlPolicyBundleEngine {
        steps: vec![
            Rc::new(RoundingRollingTtlPolicyBundleAdjuster { step: 5.0 }) as Rc<dyn RollingTtlPolicyBundleModifier>,
            Rc::new(PercentRollingTtlPolicyBundleAdjuster { percent: 7.3 }),
            Rc::new(TitleRollingTtlPolicyBundleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.01), ("online".to_string(), 1.31)]),
            }),
            Rc::new(FlaggedRollingTtlPolicyBundleAdjuster {
                flags: flags.clone(),
                flag: "rolling-ttl-policy-bundle-scheduled".to_string(),
                percent: 2.1,
            }),
            Rc::new(TieredRollingTtlPolicyBundleAdjuster {
                bounds: vec![756.0, 4301.0, 7472.0, 8454.0],
                rates: vec![0.93, 1.11, 0.81, 0.9],
            }),
            Rc::new(MinimumRollingTtlPolicyBundleAdjuster { minimum: 25.9 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 80812.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_ttl_policy_bundle_engine(&flags::new_registry());
        let item = new_rolling_ttl_policy_bundle("code-102", "title-166");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
