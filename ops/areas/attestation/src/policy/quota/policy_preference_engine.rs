use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy preference calculation.
pub trait PolicyPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePolicyPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyPreferenceAdjuster for NamePolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PolicyPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyPreferenceAdjuster for FlaggedPolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyPreferenceAdjuster {
    pub cap: f64,
}

impl PolicyPreferenceAdjuster for CapPolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyPreferenceAdjuster {
    pub step: f64,
}

impl PolicyPreferenceAdjuster for RoundingPolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyPreferenceAdjuster for TieredPolicyPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyPreferenceStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl PolicyPreferenceAdjuster for PolicyPreferenceStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyPreference, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

/// Runs the policy preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyPreferenceEngine {
    pub steps: Vec<Rc<dyn PolicyPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyPreferenceEngine {
    pub fn run(&mut self, item: &PolicyPreference) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[PolicyPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_policy_preference_engine(flags: &flags::Registry) -> PolicyPreferenceEngine {
    PolicyPreferenceEngine {
        steps: vec![
            Rc::new(NamePolicyPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.34),
                    ("north".to_string(), 1.14),
                    ("online".to_string(), 0.83),
                ]),
            }) as Rc<dyn PolicyPreferenceAdjuster>,
            Rc::new(FlaggedPolicyPreferenceAdjuster {
                flags: flags.clone(),
                flag: "policy-preference-capped".to_string(),
                percent: 1.7,
            }),
            Rc::new(CapPolicyPreferenceAdjuster { cap: 48674.0 }),
            Rc::new(RoundingPolicyPreferenceAdjuster { step: 0.05 }),
            Rc::new(TieredPolicyPreferenceAdjuster {
                bounds: vec![755.0, 6733.0],
                rates: vec![0.92, 0.93],
            }),
            Rc::new(PolicyPreferenceStartsAtOffsetAdjuster { weight: 1.82 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 52718.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_preference_engine(&flags::new_registry());
        let item = new_policy_preference("code-306", "name-544");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
