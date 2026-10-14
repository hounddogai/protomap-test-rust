use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered verifier bundle calculation.
pub trait TieredVerifierBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredVerifierBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredVerifierBundleAdjuster {
    pub cap: f64,
}

impl TieredVerifierBundleFactor for CapTieredVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredVerifierBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceTieredVerifierBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredVerifierBundleFactor for NamespaceTieredVerifierBundleAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &TieredVerifierBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredVerifierBundleAdjuster {
    pub minimum: f64,
}

impl TieredVerifierBundleFactor for MinimumTieredVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredVerifierBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredVerifierBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredVerifierBundleFactor for FlaggedTieredVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredVerifierBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredVerifierBundleAdjuster {
    pub step: f64,
}

impl TieredVerifierBundleFactor for RoundingTieredVerifierBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredVerifierBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundleTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl TieredVerifierBundleFactor for TieredVerifierBundleTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &TieredVerifierBundle, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the tiered verifier bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredVerifierBundleEngine {
    pub steps: Vec<Rc<dyn TieredVerifierBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredVerifierBundleEngine {
    pub fn compute(&mut self, item: &TieredVerifierBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredVerifierBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_verifier_bundle_engine(flags: &flags::Registry) -> TieredVerifierBundleEngine {
    TieredVerifierBundleEngine {
        steps: vec![
            Rc::new(CapTieredVerifierBundleAdjuster { cap: 42890.0 }) as Rc<dyn TieredVerifierBundleFactor>,
            Rc::new(NamespaceTieredVerifierBundleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.93)]),
            }),
            Rc::new(MinimumTieredVerifierBundleAdjuster { minimum: 17.4 }),
            Rc::new(FlaggedTieredVerifierBundleAdjuster {
                flags: flags.clone(),
                flag: "tiered-verifier-bundle-manual".to_string(),
                percent: 1.0,
            }),
            Rc::new(RoundingTieredVerifierBundleAdjuster { step: 0.1 }),
            Rc::new(TieredVerifierBundleTtlSecondsOffsetAdjuster { weight: 0.92 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 75803.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_verifier_bundle_engine(&flags::new_registry());
        let item = new_tiered_verifier_bundle("code-707", "label-638");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
