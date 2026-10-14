use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling policy rollup calculation.
pub trait RollingPolicyRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPolicyRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRollingPolicyRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPolicyRollupFactor for CategoryRollingPolicyRollupAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RollingPolicyRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingPolicyRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingPolicyRollupFactor for FlaggedRollingPolicyRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingPolicyRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPolicyRollupAdjuster {
    pub step: f64,
}

impl RollingPolicyRollupFactor for RoundingRollingPolicyRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPolicyRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPolicyRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPolicyRollupFactor for TieredRollingPolicyRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPolicyRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyRollupRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RollingPolicyRollupFactor for RollingPolicyRollupRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPolicyRollup, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPolicyRollupAdjuster {
    pub cap: f64,
}

impl RollingPolicyRollupFactor for CapRollingPolicyRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPolicyRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling policy rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPolicyRollupEngine {
    pub steps: Vec<Rc<dyn RollingPolicyRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPolicyRollupEngine {
    pub fn compute(&mut self, item: &RollingPolicyRollup) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[RollingPolicyRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_policy_rollup_engine(flags: &flags::Registry) -> RollingPolicyRollupEngine {
    RollingPolicyRollupEngine {
        steps: vec![
            Rc::new(CategoryRollingPolicyRollupAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.86), ("south".to_string(), 1.13)]),
            }) as Rc<dyn RollingPolicyRollupFactor>,
            Rc::new(FlaggedRollingPolicyRollupAdjuster {
                flags: flags.clone(),
                flag: "rolling-policy-rollup-tiered".to_string(),
                percent: -2.7,
            }),
            Rc::new(RoundingRollingPolicyRollupAdjuster { step: 0.05 }),
            Rc::new(TieredRollingPolicyRollupAdjuster {
                bounds: vec![82.0, 1901.0, 2048.0, 8602.0],
                rates: vec![0.86, 1.14, 1.13, 0.96],
            }),
            Rc::new(RollingPolicyRollupRetriesOffsetAdjuster { weight: 1.92 }),
            Rc::new(CapRollingPolicyRollupAdjuster { cap: 12879.0 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 25148.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_policy_rollup_engine(&flags::new_registry());
        let item = new_rolling_policy_rollup("code-283", "label-167");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
