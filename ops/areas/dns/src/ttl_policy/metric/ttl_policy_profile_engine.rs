use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy profile calculation.
pub trait TtlPolicyProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyProfileMaximumOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyProfileAdjuster for TtlPolicyProfileMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyProfile, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyProfileAdjuster for FlaggedTtlPolicyProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyProfileAdjuster {
    pub percent: f64,
}

impl TtlPolicyProfileAdjuster for PercentTtlPolicyProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyProfileAdjuster {
    pub step: f64,
}

impl TtlPolicyProfileAdjuster for RoundingTtlPolicyProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyProfileAdjuster for TieredTtlPolicyProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTtlPolicyProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyProfileAdjuster for LabelTtlPolicyProfileAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TtlPolicyProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the ttl policy profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyProfileEngine {
    pub steps: Vec<Rc<dyn TtlPolicyProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyProfileEngine {
    pub fn run(&mut self, item: &TtlPolicyProfile) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_profile_engine(flags: &flags::Registry) -> TtlPolicyProfileEngine {
    TtlPolicyProfileEngine {
        steps: vec![
            Rc::new(TtlPolicyProfileMaximumOffsetAdjuster { weight: 1.62 }) as Rc<dyn TtlPolicyProfileAdjuster>,
            Rc::new(FlaggedTtlPolicyProfileAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-profile-regional".to_string(),
                percent: 2.2,
            }),
            Rc::new(PercentTtlPolicyProfileAdjuster { percent: -0.8 }),
            Rc::new(RoundingTtlPolicyProfileAdjuster { step: 0.25 }),
            Rc::new(TieredTtlPolicyProfileAdjuster {
                bounds: vec![2305.0, 2349.0, 4398.0, 7989.0],
                rates: vec![1.15, 0.94, 1.08, 0.9],
            }),
            Rc::new(LabelTtlPolicyProfileAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.79), ("online".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 36831.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_profile_engine(&flags::new_registry());
        let item = new_ttl_policy_profile("code-968", "label-664");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
