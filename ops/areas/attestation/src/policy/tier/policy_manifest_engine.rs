use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the policy manifest calculation.
pub trait PolicyManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyManifestAdjuster {
    pub percent: f64,
}

impl PolicyManifestModifier for PercentPolicyManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPolicyManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyManifestModifier for LabelPolicyManifestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PolicyManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyManifestTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl PolicyManifestModifier for PolicyManifestTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyManifest, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyManifestAdjuster {
    pub cap: f64,
}

impl PolicyManifestModifier for CapPolicyManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyManifestModifier for FlaggedPolicyManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyManifestModifier for TieredPolicyManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the policy manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyManifestEngine {
    pub steps: Vec<Rc<dyn PolicyManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyManifestEngine {
    pub fn compute(&mut self, item: &PolicyManifest) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyManifestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_policy_manifest_engine(flags: &flags::Registry) -> PolicyManifestEngine {
    PolicyManifestEngine {
        steps: vec![
            Rc::new(PercentPolicyManifestAdjuster { percent: -10.3 }) as Rc<dyn PolicyManifestModifier>,
            Rc::new(LabelPolicyManifestAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.24),
                    ("retail".to_string(), 0.91),
                    ("east".to_string(), 1.2),
                ]),
            }),
            Rc::new(PolicyManifestTtlSecondsOffsetAdjuster { weight: 0.51 }),
            Rc::new(CapPolicyManifestAdjuster { cap: 4025.0 }),
            Rc::new(FlaggedPolicyManifestAdjuster {
                flags: flags.clone(),
                flag: "policy-manifest-scheduled".to_string(),
                percent: -0.3,
            }),
            Rc::new(TieredPolicyManifestAdjuster {
                bounds: vec![2989.0, 4551.0, 8404.0],
                rates: vec![1.0, 1.05, 1.05],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 53923.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_manifest_engine(&flags::new_registry());
        let item = new_policy_manifest("code-900", "label-528");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
