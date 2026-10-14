use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy allowlist bundle calculation.
pub trait LegacyAllowlistBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyAllowlistBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyAllowlistBundleAdjuster {
    pub percent: f64,
}

impl LegacyAllowlistBundleStep for PercentLegacyAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyAllowlistBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyAllowlistBundleAdjuster {
    pub step: f64,
}

impl LegacyAllowlistBundleStep for RoundingLegacyAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyAllowlistBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyAllowlistBundleAdjuster {
    pub cap: f64,
}

impl LegacyAllowlistBundleStep for CapLegacyAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyAllowlistBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyAllowlistBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyAllowlistBundleStep for FlaggedLegacyAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyAllowlistBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterLegacyAllowlistBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyAllowlistBundleStep for ClusterLegacyAllowlistBundleAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &LegacyAllowlistBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy allowlist bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistBundleEngine {
    pub steps: Vec<Rc<dyn LegacyAllowlistBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyAllowlistBundleEngine {
    pub fn compute(&mut self, item: &LegacyAllowlistBundle) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyAllowlistBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyAllowlistBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_allowlist_bundle_engine(flags: &flags::Registry) -> LegacyAllowlistBundleEngine {
    LegacyAllowlistBundleEngine {
        steps: vec![
            Rc::new(PercentLegacyAllowlistBundleAdjuster { percent: -3.8 }) as Rc<dyn LegacyAllowlistBundleStep>,
            Rc::new(RoundingLegacyAllowlistBundleAdjuster { step: 0.05 }),
            Rc::new(CapLegacyAllowlistBundleAdjuster { cap: 48756.0 }),
            Rc::new(FlaggedLegacyAllowlistBundleAdjuster {
                flags: flags.clone(),
                flag: "legacy-allowlist-bundle-tiered".to_string(),
                percent: 8.6,
            }),
            Rc::new(ClusterLegacyAllowlistBundleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.06), ("north".to_string(), 1.24)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 45054.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_allowlist_bundle_engine(&flags::new_registry());
        let item = new_legacy_allowlist_bundle("code-741", "name-92");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
